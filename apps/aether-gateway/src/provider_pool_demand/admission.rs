use super::*;
mod eligibility;

tokio::task_local! {
    static PREPARED_ADMISSION: std::sync::Mutex<Option<PreparedProviderAdmission>>;
}

pub(crate) struct PreparedProviderAdmission {
    request_id: String,
    provider_id: String,
    key_id: String,
    admission: ProviderPoolInFlightAdmission,
}

impl PreparedProviderAdmission {
    pub(crate) async fn scope<F: std::future::Future>(self, future: F) -> F::Output {
        PREPARED_ADMISSION
            .scope(std::sync::Mutex::new(Some(self)), future)
            .await
    }
}

pub(crate) async fn prepare_provider_execution_admission(
    state: &AppState,
    plan: &ExecutionPlan,
) -> Result<PreparedProviderAdmission, GatewayError> {
    let admission = acquire_provider_pool_execution_guard_unobserved(state, plan).await?;
    Ok(PreparedProviderAdmission {
        request_id: plan.request_id.clone(),
        provider_id: plan.provider_id.clone(),
        key_id: plan.key_id.clone(),
        admission,
    })
}

pub(crate) async fn acquire_provider_pool_execution_guard(
    state: &AppState,
    plan: &ExecutionPlan,
    report_context: Option<&serde_json::Value>,
) -> Result<ProviderPoolInFlightAdmission, GatewayError> {
    let admission = acquire_provider_pool_execution_guard_unobserved(state, plan).await?;
    let ProviderPoolInFlightAdmission::Acquired(guard) = admission else {
        return Ok(admission);
    };
    state.mutate_local_execution_runtime_miss_diagnostic(&plan.request_id, |diagnostic| {
        diagnostic.execution_admitted = true;
    });
    let requested_model = report_context
        .and_then(|context| context.get("model"))
        .and_then(serde_json::Value::as_str);
    let observation_id = execution_observation_request_id(&plan.request_id, report_context);
    let activity = state.execution_activity.begin(
        observation_id.as_ref(),
        &plan.provider_id,
        plan.provider_name.as_deref(),
        requested_model,
    );
    Ok(ProviderPoolInFlightAdmission::Acquired(Some(
        ProviderPoolInFlightGuard::observe_execution(guard, activity),
    )))
}

pub(crate) async fn acquire_provider_pool_execution_guard_unobserved(
    state: &AppState,
    plan: &ExecutionPlan,
) -> Result<ProviderPoolInFlightAdmission, GatewayError> {
    if let Some(admission) = PREPARED_ADMISSION
        .try_with(|prepared| {
            let mut slot = prepared.lock().unwrap_or_else(|error| error.into_inner());
            if slot.as_ref().is_some_and(|prepared| {
                prepared.request_id == plan.request_id
                    && prepared.provider_id == plan.provider_id
                    && prepared.key_id == plan.key_id
            }) {
                slot.take().map(|prepared| prepared.admission)
            } else {
                None
            }
        })
        .ok()
        .flatten()
    {
        return validate_admission(state, plan, admission).await;
    }
    let (providers, keys) = tokio::try_join!(
        state.read_provider_catalog_providers_by_ids(std::slice::from_ref(&plan.provider_id)),
        state.read_provider_catalog_keys_by_ids(std::slice::from_ref(&plan.key_id)),
    )?;
    let limits = [
        (
            PROVIDER_KEY_CONCURRENCY_GATE,
            plan.key_id.as_str(),
            keys.into_iter()
                .find(|key| key.id == plan.key_id)
                .and_then(|key| key.concurrent_limit),
        ),
        (
            "provider",
            plan.provider_id.as_str(),
            providers
                .into_iter()
                .find(|provider| provider.id == plan.provider_id)
                .and_then(|provider| provider.concurrent_limit),
        ),
    ];
    let mut gates = Vec::with_capacity(2);
    for (name, resource, limit) in limits {
        if let Some(limit) = limit
            .filter(|limit| *limit > 0)
            .and_then(|limit| usize::try_from(limit).ok())
        {
            gates.push(
                state
                    .runtime_state
                    .keyed_semaphore(name, resource, limit, RuntimeSemaphoreConfig::default())
                    .map_err(|error| GatewayError::Internal(error.to_string()))?,
            );
        }
    }
    let _queue_wait = crate::execution_runtime::StreamCandidateWatchdogProgress::current()
        .map(|progress| progress.begin_queue_wait());
    let permits = match state
        .provider_concurrency_queue
        .acquire(&gates, &plan.key_id)
        .await
    {
        Ok(permits) => permits,
        Err(RuntimeSemaphoreError::Saturated { gate, limit }) => {
            let reason = if gate == "provider" {
                "provider_concurrency_limit_reached"
            } else {
                "provider_key_concurrency_limit_reached"
            };
            return Ok(ProviderPoolInFlightAdmission::Rejected {
                reason,
                status: 429,
                message: format!("{gate} concurrency admission exhausted: {limit}"),
            });
        }
        Err(error) => return Err(GatewayError::Internal(error.to_string())),
    };
    drop(_queue_wait);
    let guard = acquire_provider_pool_in_flight_guard_with_key_limit(
        Arc::clone(&state.runtime_state),
        &plan.provider_id,
        &plan.request_id,
        plan.candidate_id.as_deref(),
        &plan.key_id,
        None,
    )
    .await
    .map_err(|error| GatewayError::Internal(error.to_string()))?;
    let mut guard = guard.unwrap_or(ProviderPoolInFlightGuard {
        kind: ProviderPoolInFlightGuardKind::Disabled,
        provider_key_permit: None,
        concurrency_permits: Vec::new(),
        observed_activity: None,
        released: false,
    });
    guard.concurrency_permits = permits;
    validate_admission(
        state,
        plan,
        ProviderPoolInFlightAdmission::Acquired(Some(guard)),
    )
    .await
}

async fn validate_admission(
    state: &AppState,
    plan: &ExecutionPlan,
    admission: ProviderPoolInFlightAdmission,
) -> Result<ProviderPoolInFlightAdmission, GatewayError> {
    let ProviderPoolInFlightAdmission::Acquired(Some(guard)) = admission else {
        return Ok(admission);
    };
    if !guard.concurrency_permits.is_empty() {
        if let Some(reason) = eligibility::current_skip_reason(state, plan).await? {
            guard.release().await;
            return Ok(ProviderPoolInFlightAdmission::Rejected {
                reason,
                status: 503,
                message: format!("upstream became unavailable during admission: {reason}"),
            });
        }
    }
    Ok(ProviderPoolInFlightAdmission::Acquired(Some(guard)))
}
