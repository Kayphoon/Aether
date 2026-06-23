use std::sync::{Arc, Mutex};

use aether_data::repository::background_tasks::InMemoryBackgroundTaskRepository;
use aether_data::repository::global_models::InMemoryGlobalModelReadRepository;
use aether_data::DataLayerError;
use aether_data_contracts::repository::background_tasks::{
    BackgroundTaskKind, BackgroundTaskStatus, StoredBackgroundTaskRun,
};
use aether_data_contracts::repository::global_models::{
    AdminGlobalModelListQuery, AdminProviderModelListQuery, GlobalModelReadRepository,
    PublicCatalogModelListQuery, PublicCatalogModelSearchQuery, PublicGlobalModelQuery,
    StoredAdminGlobalModel, StoredAdminGlobalModelPage, StoredAdminProviderModel,
    StoredProviderActiveGlobalModel, StoredProviderModelStats, StoredPublicCatalogModel,
    StoredPublicGlobalModel, StoredPublicGlobalModelPage,
};
use async_trait::async_trait;
use axum::body::Body;
use axum::routing::any;
use axum::{extract::Request, Router};
use http::StatusCode;
use serde_json::json;

use super::super::{build_router_with_state, start_server, AppState};
use crate::constants::{
    TRUSTED_ADMIN_SESSION_ID_HEADER, TRUSTED_ADMIN_USER_ID_HEADER, TRUSTED_ADMIN_USER_ROLE_HEADER,
};
use crate::data::GatewayDataState;
use crate::task_runtime::TASK_KEY_SYSTEM_DATA_EXPORT;

#[derive(Debug, Default)]
struct FailingAdminGlobalModelReadRepository {
    fallback: InMemoryGlobalModelReadRepository,
}

#[async_trait]
impl GlobalModelReadRepository for FailingAdminGlobalModelReadRepository {
    async fn list_public_models(
        &self,
        query: &PublicGlobalModelQuery,
    ) -> Result<StoredPublicGlobalModelPage, DataLayerError> {
        self.fallback.list_public_models(query).await
    }

    async fn get_public_model_by_name(
        &self,
        model_name: &str,
    ) -> Result<Option<StoredPublicGlobalModel>, DataLayerError> {
        self.fallback.get_public_model_by_name(model_name).await
    }

    async fn list_public_catalog_models(
        &self,
        query: &PublicCatalogModelListQuery,
    ) -> Result<Vec<StoredPublicCatalogModel>, DataLayerError> {
        self.fallback.list_public_catalog_models(query).await
    }

    async fn search_public_catalog_models(
        &self,
        query: &PublicCatalogModelSearchQuery,
    ) -> Result<Vec<StoredPublicCatalogModel>, DataLayerError> {
        self.fallback.search_public_catalog_models(query).await
    }

    async fn list_admin_global_models(
        &self,
        _query: &AdminGlobalModelListQuery,
    ) -> Result<StoredAdminGlobalModelPage, DataLayerError> {
        Err(DataLayerError::Sql(
            "complete backup export fixture failure".to_string(),
        ))
    }

    async fn list_admin_provider_models(
        &self,
        query: &AdminProviderModelListQuery,
    ) -> Result<Vec<StoredAdminProviderModel>, DataLayerError> {
        self.fallback.list_admin_provider_models(query).await
    }

    async fn list_admin_provider_available_source_models(
        &self,
        provider_id: &str,
    ) -> Result<Vec<StoredAdminProviderModel>, DataLayerError> {
        self.fallback
            .list_admin_provider_available_source_models(provider_id)
            .await
    }

    async fn get_admin_provider_model(
        &self,
        provider_id: &str,
        model_id: &str,
    ) -> Result<Option<StoredAdminProviderModel>, DataLayerError> {
        self.fallback
            .get_admin_provider_model(provider_id, model_id)
            .await
    }

    async fn get_admin_global_model_by_id(
        &self,
        global_model_id: &str,
    ) -> Result<Option<StoredAdminGlobalModel>, DataLayerError> {
        self.fallback
            .get_admin_global_model_by_id(global_model_id)
            .await
    }

    async fn get_admin_global_model_by_name(
        &self,
        model_name: &str,
    ) -> Result<Option<StoredAdminGlobalModel>, DataLayerError> {
        self.fallback
            .get_admin_global_model_by_name(model_name)
            .await
    }

    async fn list_admin_provider_models_by_global_model_id(
        &self,
        global_model_id: &str,
    ) -> Result<Vec<StoredAdminProviderModel>, DataLayerError> {
        self.fallback
            .list_admin_provider_models_by_global_model_id(global_model_id)
            .await
    }

    async fn list_provider_model_stats(
        &self,
        provider_ids: &[String],
    ) -> Result<Vec<StoredProviderModelStats>, DataLayerError> {
        self.fallback.list_provider_model_stats(provider_ids).await
    }

    async fn list_active_global_model_ids_by_provider_ids(
        &self,
        provider_ids: &[String],
    ) -> Result<Vec<StoredProviderActiveGlobalModel>, DataLayerError> {
        self.fallback
            .list_active_global_model_ids_by_provider_ids(provider_ids)
            .await
    }
}

fn sample_background_task_run(
    id: &str,
    task_key: &str,
    status: BackgroundTaskStatus,
    created_at_unix_secs: u64,
) -> StoredBackgroundTaskRun {
    StoredBackgroundTaskRun {
        id: id.to_string(),
        task_key: task_key.to_string(),
        kind: BackgroundTaskKind::OnDemand,
        trigger: "manual".to_string(),
        status,
        attempt: 1,
        max_attempts: 1,
        owner_instance: None,
        progress_percent: if status == BackgroundTaskStatus::Succeeded {
            100
        } else {
            0
        },
        progress_message: None,
        payload_json: Some(json!({
            "scope": "complete_backup",
            "secret": "must-not-leak"
        })),
        result_json: Some(json!({
            "exported_bytes": 512,
            "user_data": { "users": ["must-not-leak"] }
        })),
        error_message: None,
        cancel_requested: false,
        created_by: Some("admin-user-123".to_string()),
        created_at_unix_secs,
        started_at_unix_secs: Some(created_at_unix_secs),
        finished_at_unix_secs: if status == BackgroundTaskStatus::Succeeded {
            Some(created_at_unix_secs + 1)
        } else {
            None
        },
        updated_at_unix_secs: created_at_unix_secs + 1,
    }
}

#[tokio::test]
async fn gateway_lists_admin_system_backup_runs_filtered_to_data_export_task() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/api/admin/system/backup-runs",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );

    let repository = Arc::new(InMemoryBackgroundTaskRepository::seed_runs(vec![
        sample_background_task_run(
            "backup-run-1",
            TASK_KEY_SYSTEM_DATA_EXPORT,
            BackgroundTaskStatus::Succeeded,
            1_711_000_000,
        ),
        sample_background_task_run(
            "backup-run-near-match",
            "admin.system.data_export.extra",
            BackgroundTaskStatus::Succeeded,
            1_711_000_200,
        ),
        sample_background_task_run(
            "provider-delete-1",
            "admin.provider.delete",
            BackgroundTaskStatus::Succeeded,
            1_711_000_100,
        ),
    ]));
    let data_state =
        GatewayDataState::disabled().with_background_task_repository_for_tests(repository);

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_data_state_for_tests(data_state),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!(
            "{gateway_url}/api/admin/system/backup-runs?page=1&page_size=20"
        ))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    println!("backup-runs filtered response: {payload}");
    assert_eq!(payload["total"], json!(1));
    assert_eq!(payload["page"], json!(1));
    assert_eq!(payload["page_size"], json!(20));
    assert_eq!(payload["pages"], json!(1));
    assert_eq!(payload["items"][0]["id"], "backup-run-1");
    assert_eq!(payload["items"][0]["task_key"], TASK_KEY_SYSTEM_DATA_EXPORT);
    assert_eq!(payload["items"][0]["kind"], "on_demand");
    assert_eq!(payload["items"][0]["trigger"], "manual");
    assert_eq!(payload["items"][0]["status"], "succeeded");
    assert_eq!(payload["items"][0]["progress_percent"], json!(100));
    assert_eq!(payload["items"][0]["result"]["exported_bytes"], json!(512));
    assert_eq!(
        payload["items"][0]["payload"],
        json!({ "scope": "complete_backup" })
    );
    assert!(payload["items"][0]["payload"].get("secret").is_none());
    assert!(payload["items"][0]["result"].get("user_data").is_none());
    assert_eq!(payload["items"][0]["created_by"], "admin-user-123");
    assert!(payload["items"][0]["created_at"].as_str().is_some());
    assert!(payload["items"][0]["started_at"].as_str().is_some());
    assert!(payload["items"][0]["finished_at"].as_str().is_some());
    assert!(payload["items"][0]["updated_at"].as_str().is_some());
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    let empty_response = reqwest::Client::new()
        .get(format!(
            "{gateway_url}/api/admin/system/backup-runs?page=2&page_size=20"
        ))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(empty_response.status(), StatusCode::OK);
    let empty_payload: serde_json::Value =
        empty_response.json().await.expect("json body should parse");
    println!("backup-runs empty page response: {empty_payload}");
    assert_eq!(empty_payload["total"], json!(1));
    assert_eq!(empty_payload["page"], json!(2));
    assert_eq!(empty_payload["items"], json!([]));

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_records_failed_admin_system_data_export_backup_run() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/api/admin/system/data/export",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );

    let repository = Arc::new(InMemoryBackgroundTaskRepository::default());
    let global_models = Arc::new(FailingAdminGlobalModelReadRepository::default());
    let data_state = GatewayDataState::disabled()
        .with_background_task_repository_for_tests(Arc::clone(&repository))
        .with_global_model_reader(global_models);

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_data_state_for_tests(data_state),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/admin/system/data/export"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert!(response.status().is_server_error());
    let export_error: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(
        export_error["error"]["message"],
        "sql error: complete backup export fixture failure"
    );

    let runs_response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/admin/system/backup-runs"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(runs_response.status(), StatusCode::OK);
    let runs_payload: serde_json::Value =
        runs_response.json().await.expect("json body should parse");
    println!("backup-runs after failed export response: {runs_payload}");
    assert_eq!(runs_payload["total"], json!(1));
    assert_eq!(
        runs_payload["items"][0]["task_key"],
        TASK_KEY_SYSTEM_DATA_EXPORT
    );
    assert_eq!(runs_payload["items"][0]["kind"], "on_demand");
    assert_eq!(runs_payload["items"][0]["trigger"], "manual");
    assert_eq!(runs_payload["items"][0]["status"], "failed");
    assert_eq!(runs_payload["items"][0]["progress_percent"], json!(100));
    assert_eq!(
        runs_payload["items"][0]["progress_message"],
        "完整备份导出失败"
    );
    assert_eq!(
        runs_payload["items"][0]["payload"],
        json!({ "scope": "complete_backup" })
    );
    assert_eq!(runs_payload["items"][0]["result"], serde_json::Value::Null);
    assert_eq!(
        runs_payload["items"][0]["error_message"],
        "sql error: complete backup export fixture failure"
    );
    assert_eq!(runs_payload["items"][0]["created_by"], "admin-user-123");
    assert!(runs_payload["items"][0]["started_at"].as_str().is_some());
    assert!(runs_payload["items"][0]["finished_at"].as_str().is_some());
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}

#[tokio::test]
async fn gateway_handles_admin_system_data_export_records_backup_run() {
    let upstream_hits = Arc::new(Mutex::new(0usize));
    let upstream_hits_clone = Arc::clone(&upstream_hits);
    let upstream = Router::new().route(
        "/api/admin/system/data/export",
        any(move |_request: Request| {
            let upstream_hits_inner = Arc::clone(&upstream_hits_clone);
            async move {
                *upstream_hits_inner.lock().expect("mutex should lock") += 1;
                (StatusCode::OK, Body::from("unexpected upstream hit"))
            }
        }),
    );

    let repository = Arc::new(InMemoryBackgroundTaskRepository::default());
    let data_state = GatewayDataState::disabled()
        .with_background_task_repository_for_tests(Arc::clone(&repository));

    let (upstream_url, upstream_handle) = start_server(upstream).await;
    let gateway = build_router_with_state(
        AppState::new()
            .expect("gateway should build")
            .with_data_state_for_tests(data_state),
    );
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/admin/system/data/export"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let export_payload: serde_json::Value = response.json().await.expect("json body should parse");
    assert_eq!(export_payload["version"], "1.0");
    assert!(export_payload["exported_at"].as_str().is_some());

    let runs_response = reqwest::Client::new()
        .get(format!("{gateway_url}/api/admin/system/backup-runs"))
        .header(crate::constants::GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(runs_response.status(), StatusCode::OK);
    let runs_payload: serde_json::Value =
        runs_response.json().await.expect("json body should parse");
    println!("backup-runs after export response: {runs_payload}");
    assert_eq!(runs_payload["total"], json!(1));
    assert_eq!(
        runs_payload["items"][0]["task_key"],
        TASK_KEY_SYSTEM_DATA_EXPORT
    );
    assert_eq!(runs_payload["items"][0]["kind"], "on_demand");
    assert_eq!(runs_payload["items"][0]["trigger"], "manual");
    assert_eq!(runs_payload["items"][0]["status"], "succeeded");
    assert_eq!(runs_payload["items"][0]["progress_percent"], json!(100));
    assert_eq!(
        runs_payload["items"][0]["progress_message"],
        "完整备份导出完成"
    );
    assert_eq!(runs_payload["items"][0]["created_by"], "admin-user-123");
    assert_eq!(
        runs_payload["items"][0]["payload"],
        json!({ "scope": "complete_backup" })
    );
    assert!(runs_payload["items"][0]["result"]["exported_bytes"]
        .as_u64()
        .is_some_and(|value| value > 0));
    assert!(runs_payload["items"][0]["result"]
        .get("config_data")
        .is_none());
    assert!(runs_payload["items"][0]["result"]
        .get("user_data")
        .is_none());
    assert_eq!(*upstream_hits.lock().expect("mutex should lock"), 0);

    gateway_handle.abort();
    upstream_handle.abort();
}
