use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use lru::LruCache;
use std::sync::Arc;
use std::num::NonZeroUsize;

const MEMORY_LIMIT_BYTES: usize = 1_400_000_000;

struct ShardedCache {
    data: Arc<Mutex<LruCache<String, String>>>,
}

impl ShardedCache {
    fn new(capacity: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap()))),
        }
    }

    async fn set(&self, key: String, value: String) {
        let mut cache = self.data.lock().await;
        cache.put(key, value);
    }

    async fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.data.lock().await;
        cache.get(key).cloned()
    }
}

async fn handle_client(mut stream: TcpStream, cache: Arc<ShardedCache>) {
    let mut buffer = [0; 512];

    if let Err(e) = stream.set_nodelay(true) {
        eprintln!("Failed to set TCP_NODELAY: {}", e);
        return;
    }

    while let Ok(bytes_read) = stream.read(&mut buffer).await {
        if bytes_read == 0 {
            break;
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let parts: Vec<&str> = request.split("\r\n").collect();

        if parts.len() < 4 {
            let _ = stream.write_all(b"-ERR Invalid command\r\n").await;
            continue;
        }

        match parts[0] {
            "*3" if parts[1] == "$3" && parts[2] == "SET" => {
                if parts.len() >= 6 {
                    let key = parts[4].to_string();
                    let value = parts[6].to_string();
                    cache.set(key, value).await;
                    let _ = stream.write_all(b"+OK\r\n").await;
                } else {
                    let _ = stream.write_all(b"-ERR Invalid SET format\r\n").await;
                }
            }
            "*2" if parts[1] == "$3" && parts[2] == "GET" => {
                if parts.len() >= 4 {
                    let key = parts[4].to_string();
                    if let Some(value) = cache.get(&key).await {
                        let response = format!("${}\r\n{}\r\n", value.len(), value);
                        let _ = stream.write_all(response.as_bytes()).await;
                    } else {
                        let _ = stream.write_all(b"$-1\r\n").await;
                    }
                } else {
                    let _ = stream.write_all(b"-ERR Invalid GET format\r\n").await;
                }
            }
            _ => {
                let _ = stream.write_all(b"-ERR unknown command\r\n").await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let cache = Arc::new(ShardedCache::new(100_000)); // Holds 100,000 entries before eviction
    let listener = TcpListener::bind("0.0.0.0:7171").await.expect("Failed to bind to port 7171");

    println!("🚀 RESP Server running on port 7171");

    let cache_eviction = Arc::clone(&cache);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let mut cache = cache_eviction.data.lock().await;
            if cache.len() > 90_000 {
                cache.pop_lru(); // Evict the least recently used item
            }
        }
    });

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let cache_clone = Arc::clone(&cache);
                tokio::spawn(handle_client(stream, cache_clone));
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
}
