use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::{Arc, Mutex}};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingMessage {
    message_type: String,
    data: Option<String>,
    data_array: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutgoingMessage {
    message_type: String,
    data: Option<String>,
    data_array: Option<Vec<String>>,
}

type Users = Arc<Mutex<HashMap<SocketAddr, String>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let users: Users = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("Listening on ws://127.0.0.1:2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        let bcast_tx = bcast_tx.clone();
        let users = users.clone();

        tokio::spawn(async move {
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, bcast_tx, users).await
        });
    }
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    users: Users,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            Some(msg) = ws_stream.next() => {
                if let Ok(msg) = msg {
                    if let Some(text) = msg.as_text() {
                        if let Ok(parsed) = serde_json::from_str::<IncomingMessage>(text) {
                            match parsed.message_type.as_str() {
                                "register" => {
                                    if let Some(nick) = parsed.data {
                                        users.lock().unwrap().insert(addr, nick.clone());
                                        let all_users = users.lock().unwrap().values().cloned().collect();
                                        let out = OutgoingMessage {
                                            message_type: "users".into(),
                                            data: None,
                                            data_array: Some(all_users),
                                        };
                                        bcast_tx.send(serde_json::to_string(&out).unwrap())?;
                                    }
                                }
                                "typing" => {
                                    if let Some(nick) = parsed.data {
                                        let out = OutgoingMessage {
                                            message_type: "typing".into(),
                                            data: Some(nick),
                                            data_array: None,
                                        };
                                        bcast_tx.send(serde_json::to_string(&out).unwrap())?;
                                    }
                                }
                                "message" => {
                                    if let Some(msg_text) = parsed.data {
                                        let nick = users.lock().unwrap()
                                            .get(&addr)
                                            .cloned()
                                            .unwrap_or("anonymous".into());
                                        let payload = serde_json::json!({
                                            "from": nick,
                                            "message": msg_text,
                                            "time": chrono::Utc::now().timestamp_millis()
                                        });
                                        let out = OutgoingMessage {
                                            message_type: "message".into(),
                                            data: Some(payload.to_string()),
                                            data_array: None,
                                        };
                                        bcast_tx.send(serde_json::to_string(&out).unwrap())?;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            Ok(msg) = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg)).await?;
            }
        }
    }
}
