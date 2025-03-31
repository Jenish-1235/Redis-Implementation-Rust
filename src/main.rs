use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use bytes::{Bytes, BytesMut, BufMut};
use std::{sync::Arc, str};
use tokio::sync::Mutex;
use std::num::NonZeroUsize;

const MEMORY_LIMIT_BYTES: usize = 1_400_000_000; // 1.4GB limit

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    key: String,
    value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    status: &'static str,
    message: &'static str,
    key: Option<String>,
    value: Option<String>,
}

struct ShardedCache {
    data: Arc<Mutex<LruCache<String, Bytes>>>,
}

impl ShardedCache {
    fn new(capacity: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap()))),
        }
    }

    async fn set(&self, key: String, value: Bytes) {
        let mut cache = self.data.lock().await;
        cache.put(key, value);
    }

    async fn get(&self, key: &str) -> Option<Bytes> {
        let mut cache = self.data.lock().await;
        cache.get(key).cloned()
    }
}

#[tokio::main]
async fn main() {
    let store = Arc::new(ShardedCache::new(100_000)); // 100,000 entries before eviction
    let listener = TcpListener::bind("0.0.0.0:7171").await.unwrap();
    println!("🚀 TCP Server running on 0.0.0.0:7171");

    let store_eviction = Arc::clone(&store);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let mut cache = store_eviction.data.lock().await;
            if cache.len() > 90_000 {
                cache.pop_lru(); // Evict the least recently used item
            }
        }
    });

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let store = store.clone();

        tokio::spawn(async move {
            let mut buffer = BytesMut::with_capacity(1024);

            if let Err(e) = socket.set_nodelay(true) {
                eprintln!("Failed to set TCP_NODELAY: {}", e);
                return;
            }

            loop {
                buffer.clear();
                match socket.read_buf(&mut buffer).await {
                    Ok(0) => break, // Connection closed
                    Ok(_) => {
                        if let Ok(req_str) = str::from_utf8(&buffer) {
                            match serde_json::from_str::<Request>(req_str) {
                                Ok(request) => {
                                    let response = if let Some(value) = request.value {
                                        store.set(request.key.clone(), Bytes::from(value)).await;
                                        Response {
                                            status: "OK",
                                            message: "Key inserted/updated successfully.",
                                            key: None,
                                            value: None,
                                        }
                                    } else {
                                        match store.get(&request.key).await {
                                            Some(value) => Response {
                                                status: "OK",
                                                message: "",
                                                key: Some(request.key.clone()),
                                                value: Some(String::from_utf8_lossy(&value).to_string()),
                                            },
                                            None => Response {
                                                status: "OK",
                                                message: "Key not found.",
                                                key: None,
                                                value: None,
                                            },
                                        }
                                    };
                                    let response_str = serde_json::to_string(&response).unwrap();
                                    let _ = socket.write_all(response_str.as_bytes()).await;
                                }
                                Err(_) => {
                                    let error_response = Response {
                                        status: "ERROR",
                                        message: "Invalid request format.",
                                        key: None,
                                        value: None,
                                    };
                                    let error_str = serde_json::to_string(&error_response).unwrap();
                                    let _ = socket.write_all(error_str.as_bytes()).await;
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
