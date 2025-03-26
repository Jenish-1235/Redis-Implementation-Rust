use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use dashmap::DashMap;
use std::{convert::Infallible, sync::Arc};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct PutRequest {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct GetResponse {
    status: String,
    key: String,
    value: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

type Store = Arc<DashMap<String, String>>;

async fn handle_request(req: Request<Body>, store: Store) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/put") => {
            let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let data: PutRequest = serde_json::from_slice(&bytes).unwrap();
            store.insert(data.key, data.value);

            let response = Response::new(Body::from(r#"{"status": "OK", "message": "Key inserted/updated successfully."}"#));
            Ok(response)
        }
        (&Method::GET, "/get") => {
            let query = req.uri().query().unwrap_or("");
            let key = query.split('=').nth(1).unwrap_or("");

            let value = store.get(key).map(|v| v.value().clone());
            if let Some(val) = value {
                let response = Response::new(Body::from(serde_json::to_string(&GetResponse {
                    status: "OK".to_string(),
                    key: key.to_string(),
                    value: Some(val),
                }).unwrap()));
                Ok(response)
            } else {
                let response = Response::new(Body::from(serde_json::to_string(&ErrorResponse {
                    status: "ERROR".to_string(),
                    message: "Key not found.".to_string(),
                }).unwrap()));
                Ok(response)
            }
        }
        _ => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()),
    }
}

#[tokio::main]
async fn main() {
    let store = Arc::new(DashMap::new());

    let service = make_service_fn(move |_| {
        let store = store.clone();
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, store.clone()))) }
    });

    let server = Server::bind(&([0, 0, 0, 0], 7171).into()).serve(service);

    println!("🚀 Server running on port 7171...");
    server.await.unwrap();
}
