use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, str};

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    key: String,
    value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    status: String,
    message: String,
    key: Option<String>,
    value: Option<String>,
}

#[tokio::main]
async fn main() {
    let store = Arc::new(DashMap::new());
    let listener = TcpListener::bind("0.0.0.0:7171").await.unwrap();
    println!("🚀 TCP Server running on 0.0.0.0:7171");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let store = store.clone();

        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];
            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => break,  // Connection closed
                    Ok(size) => {
                        if let Ok(req_str) = str::from_utf8(&buffer[..size]) {
                            match serde_json::from_str::<Request>(req_str) {
                                Ok(request) => {
                                    let response = if let Some(value) = request.value {
                                        store.insert(request.key.clone(), value);
                                        Response {
                                            status: "OK".to_string(),
                                            message: "Key inserted/updated successfully.".to_string(),
                                            key: None,
                                            value: None,
                                        }
                                    } else {
                                        match store.get(&request.key) {
                                            Some(value) => Response {
                                                status: "OK".to_string(),
                                                message: "".to_string(),
                                                key: Some(request.key.clone()),
                                                value: Some(value.clone()),
                                            },
                                            None => Response {
                                                status: "OK".to_string(),
                                                message: "Key not found.".to_string(),
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
                                        status: "ERROR".to_string(),
                                        message: "Invalid request format.".to_string(),
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
