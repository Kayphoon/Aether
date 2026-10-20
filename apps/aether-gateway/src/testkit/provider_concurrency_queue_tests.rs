use super::*;
use axum::{body::Body, routing::any, Json, Router};
use http::{Response, StatusCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;

const KEY: &str = "key-openai-chat-pressure-0";

fn queue_state(base_url: String, timeout: Duration, capacity: usize) -> AppState {
    let config = OpenAiChatPressureStateConfig::new(vec![base_url]);
    let auth = Arc::new(InMemoryAuthApiKeySnapshotRepository::seed(vec![(
        Some(hash_api_key(&config.client_api_key)),
        openai_chat_pressure_auth_snapshot(&config),
    )]));
    let mut keys = openai_chat_pressure_keys(&config).unwrap();
    keys[0].concurrent_limit = Some(1);
    let catalog = Arc::new(InMemoryProviderCatalogReadRepository::seed(
        vec![openai_chat_pressure_provider()],
        openai_chat_pressure_endpoints(&config),
        keys,
    ));
    let data = GatewayDataState::with_auth_candidate_selection_provider_catalog_and_request_candidate_repository_for_tests(
        auth,
        Arc::new(InMemoryMinimalCandidateSelectionReadRepository::seed(openai_chat_pressure_candidates(&config))),
        catalog,
        Arc::new(InMemoryRequestCandidateRepository::default()),
        DEVELOPMENT_ENCRYPTION_KEY,
    );
    let mut state = AppState::new().unwrap().with_data_state_for_tests(data);
    state.provider_concurrency_queue = Arc::new(
        crate::provider_concurrency_queue::ProviderConcurrencyQueue::new(timeout, capacity),
    );
    state.upstream_execution_gate = Some(Arc::new(aether_runtime::ConcurrencyGate::new(
        "gateway_upstream_execution",
        2,
    )));
    state
}

async fn wait_until_queued(state: &AppState) {
    tokio::time::timeout(Duration::from_secs(3), async {
        while !state.provider_concurrency_queue.is_waiting(KEY) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("request must reach provider queue");
}

async fn request(url: String, stream: bool) -> (StatusCode, String) {
    let response = reqwest::Client::new().post(format!("{url}/v1/chat/completions"))
        .bearer_auth("sk-aether-openai-chat-pressure")
        .json(&serde_json::json!({"model": "gpt-5", "messages": [{"role": "user", "content": "hello"}], "stream": stream}))
        .send().await.unwrap();
    (response.status(), response.text().await.unwrap())
}

fn response(stream: bool) -> Response<Body> {
    if stream {
        Response::builder().header("content-type", "text/event-stream").body(Body::from(
            "data: {\"id\":\"chatcmpl-queue\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-5-upstream\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"queued\"},\"finish_reason\":null}]}\n\ndata: {\"id\":\"chatcmpl-queue\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-5-upstream\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n",
        )).unwrap()
    } else {
        Response::builder().header("content-type", "application/json").body(Body::from(serde_json::json!({
            "id": "chatcmpl-queue", "object": "chat.completion", "model": "gpt-5-upstream",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "queued"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        }).to_string())).unwrap()
    }
}

#[test]
fn provider_concurrency_queue_http_sync_and_stream_resume_after_release() {
    crate::tests::run_async_test_on_large_stack(
        "provider-queue-http",
        16 * 1024 * 1024,
        || async {
            for stream in [false, true] {
                let entered = Arc::new(Semaphore::new(0));
                let release = Arc::new(Semaphore::new(0));
                let hits = Arc::new(AtomicUsize::new(0));
                let app = Router::new().route(
                    "/chat/completions",
                    any({
                        let entered = entered.clone();
                        let release = release.clone();
                        let hits = hits.clone();
                        move || {
                            let entered = entered.clone();
                            let release = release.clone();
                            let hits = hits.clone();
                            async move {
                                hits.fetch_add(1, Ordering::SeqCst);
                                entered.add_permits(1);
                                if stream {
                                    let body = async_stream::stream! {
                                        yield Ok::<_, std::io::Error>(bytes::Bytes::from_static(b"data: {\"id\":\"chatcmpl-queue\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-5-upstream\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"queued\"},\"finish_reason\":null}]}\n\n"));
                                        release.acquire().await.unwrap().forget();
                                        yield Ok(bytes::Bytes::from_static(b"data: {\"id\":\"chatcmpl-queue\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-5-upstream\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n"));
                                    };
                                    return Response::builder().header("content-type", "text/event-stream").body(Body::from_stream(body)).unwrap();
                                }
                                release.acquire().await.unwrap().forget();
                                response(stream)
                            }
                        }
                    }),
                );
                let (provider_url, provider) = crate::tests::start_server(app).await;
                let state = queue_state(provider_url, Duration::from_secs(3), 2);
                let (url, gateway) =
                    crate::tests::start_server(crate::build_router_with_state(state.clone())).await;
                let first = tokio::spawn(request(url.clone(), stream));
                tokio::time::timeout(Duration::from_secs(3), entered.acquire())
                    .await
                    .unwrap()
                    .unwrap()
                    .forget();
                let second = tokio::spawn(request(url, stream));
                wait_until_queued(&state).await;
                assert_eq!(hits.load(Ordering::SeqCst), 1);
                assert!(
                    state
                        .upstream_execution_gate
                        .as_ref()
                        .unwrap()
                        .snapshot()
                        .in_flight
                        <= 1
                );
                release.add_permits(2);
                for task in [first, second] {
                    let (status, body) = tokio::time::timeout(Duration::from_secs(5), task)
                        .await
                        .unwrap()
                        .unwrap();
                    assert_eq!(status, StatusCode::OK, "{body}");
                    assert!(body.contains("queued"), "{body}");
                }
                assert_eq!(hits.load(Ordering::SeqCst), 2);
                gateway.abort();
                provider.abort();
            }
        },
    );
}

#[test]
fn provider_concurrency_queue_http_full_and_timeout_return_429() {
    crate::tests::run_async_test_on_large_stack(
        "provider-queue-http-429",
        16 * 1024 * 1024,
        || async {
            let (provider_url, provider) = crate::tests::start_server(Router::new().route(
                "/chat/completions",
                any(|| async { Json(serde_json::json!({})) }),
            ))
            .await;
            let state = queue_state(provider_url, Duration::from_millis(400), 1);
            let gate = state
                .runtime_state
                .keyed_semaphore(
                    "provider_key",
                    KEY,
                    1,
                    aether_runtime_state::RuntimeSemaphoreConfig::default(),
                )
                .unwrap();
            let held = gate.try_acquire().await.unwrap();
            let (url, gateway) =
                crate::tests::start_server(crate::build_router_with_state(state.clone())).await;
            let waiting = tokio::spawn(request(url.clone(), false));
            wait_until_queued(&state).await;
            assert_eq!(
                state
                    .upstream_execution_gate
                    .as_ref()
                    .unwrap()
                    .snapshot()
                    .in_flight,
                0
            );
            let (status, body) = request(url, false).await;
            assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
            let (status, body) = waiting.await.unwrap();
            assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
            assert_eq!(gate.snapshot().await.unwrap().in_flight, 1);
            held.release().await.unwrap();
            gateway.abort();
            provider.abort();
        },
    );
}

#[test]
fn provider_concurrency_queue_rechecks_disabled_key_before_upstream() {
    crate::tests::run_async_test_on_large_stack(
        "provider-queue-disabled",
        16 * 1024 * 1024,
        || async {
            let hits = Arc::new(AtomicUsize::new(0));
            let (provider_url, provider) = crate::tests::start_server(Router::new().route(
                "/chat/completions",
                any({
                    let hits = hits.clone();
                    move || {
                        let hits = hits.clone();
                        async move {
                            hits.fetch_add(1, Ordering::SeqCst);
                            response(false)
                        }
                    }
                }),
            ))
            .await;
            let state = queue_state(provider_url, Duration::from_secs(3), 1);
            let gate = state
                .runtime_state
                .keyed_semaphore(
                    "provider_key",
                    KEY,
                    1,
                    aether_runtime_state::RuntimeSemaphoreConfig::default(),
                )
                .unwrap();
            let held = gate.try_acquire().await.unwrap();
            let (url, gateway) =
                crate::tests::start_server(crate::build_router_with_state(state.clone())).await;
            let waiting = tokio::spawn(request(url, false));
            wait_until_queued(&state).await;
            let mut key = state
                .read_provider_catalog_keys_by_ids(&[KEY.to_string()])
                .await
                .unwrap()
                .remove(0);
            key.is_active = false;
            assert!(state
                .update_provider_catalog_key(&key)
                .await
                .unwrap()
                .is_some());
            held.release().await.unwrap();
            let (status, body) = waiting.await.unwrap();
            assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
            assert_eq!(hits.load(Ordering::SeqCst), 0);
            assert_eq!(gate.snapshot().await.unwrap().in_flight, 0);
            gateway.abort();
            provider.abort();
        },
    );
}
