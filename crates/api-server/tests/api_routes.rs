use std::sync::Arc;

use api_server::ApiServer;
use axum::{body::Body, http::Request};
use browser_core::TabIPManager;
use hyper::{body::to_bytes, StatusCode};
use serde::Deserialize;
use tokio::sync::Mutex;
use tower::ServiceExt; // for oneshot
use virtual_ip::demo_generator;

#[derive(Debug, Deserialize)]
struct TabResponse {
    tab_id: String,
    ip: String,
    country_code: String,
}

#[derive(Debug, Deserialize)]
struct VirtualIPResponse {
    ip: String,
    country_code: String,
}

#[derive(Debug, Deserialize)]
struct ValidationResponse {
    overall_pass: bool,
}

#[tokio::test]
async fn create_list_rotate_validate_tab() {
    // Arrange app
    let generator = demo_generator();
    let ip_gen = Arc::new(generator.clone());
    let tab_manager = Arc::new(Mutex::new(TabIPManager::new(generator)));
    let server = Arc::new(ApiServer::new(tab_manager, ip_gen));
    let app = server.router().await;

    // Create tab (US)
    let req = Request::builder()
        .method("POST")
        .uri("/api/tabs")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"country_code":"US"}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let created: TabResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(created.country_code, "US");

    // List tabs
    let req = Request::builder()
        .method("GET")
        .uri("/api/tabs")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let listed: Vec<TabResponse> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(listed.len(), 1);

    // Rotate IP
    let rotate_uri = format!("/api/tabs/{}/rotate-ip", created.tab_id);
    let req = Request::builder()
        .method("POST")
        .uri(&rotate_uri)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"new_country":null}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let rotated: VirtualIPResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(rotated.country_code, "US");
    assert_ne!(rotated.ip, created.ip);

    // Validate IP
    let validate_uri = format!("/api/tabs/{}/validate", created.tab_id);
    let req = Request::builder()
        .method("GET")
        .uri(&validate_uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    let validation: ValidationResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert!(validation.overall_pass);
}
