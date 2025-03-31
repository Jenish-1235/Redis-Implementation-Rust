use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task,
};
use dashmap::DashMap;
use std::{sync::Arc, net::SocketAddr};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let store = Arc::new(DashMap::new());
    let listener = TcpListener::bind("0.0.0.0:7171").await.unwrap();
    println!("🚀 RESP Server running on 0.0.0.0:7171");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let store = store.clone();

        task::spawn(handle_client(socket, store, addr));
    }
}

async fn handle_client(mut socket: TcpStream, store: Arc<DashMap<String, String>>, addr: SocketAddr) {
    // Enable TCP_NODELAY
    socket.set_nodelay(true).unwrap();

    let mut buffer = [0; 1024];
    while let Ok(size) = socket.read(&mut buffer).await {
        if size == 0 {
            break;
        }

        let request = std::str::from_utf8(&buffer[..size]).unwrap_or("");
        let response = process_request(request, &store);
        socket.write_all(response.as_bytes()).await.unwrap();
    }

    println!("🔌 Client disconnected: {}", addr);
}

fn process_request(request: &str, store: &DashMap<String, String>) -> String {
    let parts: Vec<&str> = request.trim().split_whitespace().collect();

    if parts.is_empty() {
        return "-ERROR Invalid command\r\n".to_string();
    }

    match parts[0] {
        "SET" if parts.len() == 3 => {
            store.insert(parts[1].to_string(), parts[2].to_string());
            "+OK\r\n".to_string()
        }
        "GET" if parts.len() == 2 => {
            match store.get(parts[1]) {
                Some(value) => format!("${}\r\n{}\r\n", value.len(), value),
                None => "$-1\r\n".to_string(),
            }
        }
        _ => "-ERROR Unknown command\r\n".to_string(),
    }
}
