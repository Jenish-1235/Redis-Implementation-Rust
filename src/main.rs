use axum::{routing::{get, post}, Json, Router, extract::{Query, State}};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use hyper::StatusCode;

// Define request and response structures
#[derive(Deserialize)]
struct KeyValue {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct Response {
    status: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

// In-memory key-value store
type Store = Arc<DashMap<String, String>>;

// Handle POST requests (Insert/Update Key-Value)
async fn insert_kv(State(store): State<Store>, Json(payload): Json<KeyValue>) -> (StatusCode, Json<Response>) {
    if payload.key.len() > 256 || payload.value.len() > 256 {
        return (
            StatusCode::BAD_REQUEST,
            Json(Response {
                status: "ERROR".to_string(),
                message: "Key or value exceeds 256 characters.".to_string(),
                key: None,
                value: None,
            }),
        );
    }
    store.insert(payload.key.clone(), payload.value.clone());
    (StatusCode::OK, Json(Response {
        status: "OK".to_string(),
        message: "Key inserted/updated successfully.".to_string(),
        key: None,
        value: None,
    }))
}

// Handle GET requests (Retrieve Value by Key)
async fn get_kv(State(store): State<Store>, Query(params): Query<std::collections::HashMap<String, String>>) -> (StatusCode, Json<Response>) {
    if let Some(key) = params.get("key") {
        if let Some(value) = store.get(key) {
            return (StatusCode::OK, Json(Response {
                status: "OK".to_string(),
                message: "".to_string(),
                key: Some(key.clone()),
                value: Some(value.clone()),
            }));
        }
        return (StatusCode::OK, Json(Response {
            status: "OK".to_string(),
            message: "Key not found.".to_string(),
            key: None,
            value: None,
        }));
    }
    (StatusCode::BAD_REQUEST, Json(Response {
        status: "ERROR".to_string(),
        message: "Missing 'key' parameter.".to_string(),
        key: None,
        value: None,
    }))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let store = Arc::new(DashMap::new());
    let app = Router::new()
        .route("/set", post(insert_kv))
        .route("/get", get(get_kv))
        .with_state(store.clone());

    let listener = TcpListener::bind("0.0.0.0:7171").await.unwrap();
    println!("🚀 HTTP Server running on 0.0.0.0:7171 with Tokio multi-threaded runtime");
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
