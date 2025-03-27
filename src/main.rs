use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use dashmap::DashMap;
use sysinfo::{System};

const MEMORY_LIMIT_BYTES: usize = 1_400_000_000;

struct ShardedCache {
    data: Arc<DashMap<String, String>>,
}

impl ShardedCache {
    fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    fn set(&self, key: String, value: String) {
        self.data.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).map(|v| v.clone())
    }

    fn evict_if_needed(&self) {
        let mut sys = System::new();
        sys.refresh_memory();

        if sys.available_memory() < MEMORY_LIMIT_BYTES as u64 {
            self.data.clear();
        }
    }
}

fn handle_client(mut stream: TcpStream, cache: Arc<ShardedCache>) {
    let mut buffer = [0; 512];

    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let parts: Vec<&str> = request.split("\r\n").collect();

        if parts.len() < 4 {
            let _ = stream.write_all(b"-ERR Invalid command\r\n");
            continue;
        }

        match parts[0] {
            "*3" if parts[1] == "$3" && parts[2] == "SET" => {
                if parts.len() >= 6 {
                    let key = parts[4].to_string();
                    let value = parts[6].to_string();
                    cache.set(key, value);
                    let _ = stream.write_all(b"+OK\r\n");
                } else {
                    let _ = stream.write_all(b"-ERR Invalid SET format\r\n");
                }
            }
            "*2" if parts[1] == "$3" && parts[2] == "GET" => {
                if parts.len() >= 4 {
                    let key = parts[4].to_string();
                    if let Some(value) = cache.get(&key) {
                        let response = format!("${}\r\n{}\r\n", value.len(), value);
                        let _ = stream.write_all(response.as_bytes());
                    } else {
                        let _ = stream.write_all(b"$-1\r\n");
                    }
                } else {
                    let _ = stream.write_all(b"-ERR Invalid GET format\r\n");
                }
            }
            _ => {
                let _ = stream.write_all(b"-ERR unknown command\r\n");
            }
        }
    }
}

fn main() {
    let cache = Arc::new(ShardedCache::new());

    let listener = TcpListener::bind("0.0.0.0:7171").expect("Failed to bind to port 7171");
    println!("RESP Server running on port 7171");

    let cache_eviction = Arc::clone(&cache);

    thread::spawn(move || {
        let mut last_eviction = Instant::now();
        loop {
            if last_eviction.elapsed() >= Duration::from_secs(10) {
                cache_eviction.evict_if_needed();
                last_eviction = Instant::now();
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    for stream in listener.incoming() {
        let cache_clone = Arc::clone(&cache);

        thread::spawn(move || {
            if let Ok(stream) = stream {
                handle_client(stream, cache_clone);
            }
        });
    }
}
