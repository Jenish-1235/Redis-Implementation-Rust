use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use dashmap::DashMap;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use std::{sync::Arc, num::NonZeroUsize, time::Instant};
use std::net::SocketAddr;
use lru::LruCache;
use std::sync::atomic::{AtomicUsize, Ordering};
use serde_json::json;
use socket2::SockAddr;

const SHARD_COUNT: usize = 32;
const MAX_MEMORY_BYTES: usize = 1_400_000_000; // 70% of 2GB
static MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);

type Store = Arc<[DashMap<String, (String, Instant)>; SHARD_COUNT]>;

struct Cache {
    store: Store,
    lru: Mutex<LruCache<String, ()>>,  // Now protected with a Mutex
}

impl Cache {
    fn new() -> Self {
        let mut shards = Vec::with_capacity(SHARD_COUNT);
        for _ in 0..SHARD_COUNT {
            shards.push(DashMap::new());
        }

        Self {
            store: Arc::new(shards.try_into().unwrap()),
            lru: Mutex::new(LruCache::new(NonZeroUsize::new(100_000).unwrap())),
        }
    }

    async fn check_memory(&self) {
        if MEMORY_USAGE.load(Ordering::Relaxed) > MAX_MEMORY_BYTES {
            self.evict_entries().await;
        }
    }

    async fn evict_entries(&self) {
        let mut lru = self.lru.lock().await; // Acquire lock
        while MEMORY_USAGE.load(Ordering::Relaxed) > MAX_MEMORY_BYTES * 9 / 10 {
            if let Some((key, _)) = lru.pop_lru() {
                let shard_idx = fxhash::hash(&key) % SHARD_COUNT;
                if let Some((_, v)) = self.store[shard_idx].remove(&key) {
                    MEMORY_USAGE.fetch_sub(key.len() + v.0.len(), Ordering::Relaxed);
                }
            }
        }
    }
}

async fn handle_request(req: Request<Body>, cache: Arc<Cache>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/put") => handle_put(req, cache).await,
        (&Method::GET, "/get") => handle_get(req, cache).await,
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}

async fn handle_put(req: Request<Body>, cache: Arc<Cache>) -> Result<Response<Body>, hyper::Error> {
    let body = hyper::body::to_bytes(req.into_body()).await?;
    let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let key = data.get("key").and_then(|v| v.as_str()).unwrap_or_default();
    let value = data.get("value").and_then(|v| v.as_str()).unwrap_or_default();

    if key.len() > 256 || value.len() > 256 {
        return Ok(bad_request("Key/value exceeds 256 characters"));
    }

    let shard_idx = fxhash::hash(key) % SHARD_COUNT;

    let size_diff = match cache.store[shard_idx].get(key) {
        Some(entry) => value.len() - entry.value().0.len(),
        None => key.len() + value.len(),
    };

    MEMORY_USAGE.fetch_add(size_diff, Ordering::Relaxed);
    cache.store[shard_idx].insert(key.to_string(), (value.to_string(), Instant::now()));

    {
        let mut lru = cache.lru.lock().await; // Lock LRU cache
        lru.put(key.to_string(), ());
    }

    cache.check_memory().await; // Now properly handled as async

    Ok(build_response(StatusCode::OK, json!({"status": "OK", "message": "Success"})))
}

async fn handle_get(req: Request<Body>, cache: Arc<Cache>) -> Result<Response<Body>, hyper::Error> {
    let query = req.uri().query().unwrap_or("");
    let key = query.split('=').nth(1).unwrap_or("").to_string();

    let shard_idx = fxhash::hash(&key) % SHARD_COUNT;

    match cache.store[shard_idx].get(&key) {
        Some(entry) => {
            let value = &entry.value().0;
            Ok(build_response(
                StatusCode::OK,
                json!({"status": "OK", "key": key, "value": value}),
            ))
        }
        None => Ok(build_response(
            StatusCode::NOT_FOUND,
            json!({"status": "ERROR", "message": "Key not found"}),
        )),
    }
}

fn build_response(status: StatusCode, body: serde_json::Value) -> Response<Body> {
    let body_str = body.to_string();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Content-Length", body_str.len())
        .header("Connection", "keep-alive")
        .body(Body::from(body_str))
        .unwrap()
}

fn bad_request(message: &str) -> Response<Body> {
    build_response(
        StatusCode::BAD_REQUEST,
        json!({"status": "ERROR", "message": message}),
    )
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    let cache = Arc::new(Cache::new());

    let service = make_service_fn(move |_| {
        let cache = cache.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_request(req, cache.clone())
            }))
        }
    });

    let addr: SocketAddr = ([10, 51, 2, 34], 7171).into();
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("🚀 Ultra-fast key-value store running on port 7171");

    Server::from_tcp(listener.into_std().unwrap())
        .unwrap()
        .serve(service)
        .await
        .unwrap();
}
