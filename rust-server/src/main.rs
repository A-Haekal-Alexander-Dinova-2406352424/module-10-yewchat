use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, channel};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Debug, Deserialize, Serialize)]
struct ChatMessage {
    sender: String,
    text: String,
}

fn normalize_payload(raw_text: &str, addr: SocketAddr) -> String {
    let message = serde_json::from_str::<ChatMessage>(raw_text).unwrap_or_else(|_| ChatMessage {
        sender: format!("Rust server ({addr})"),
        text: raw_text.to_string(),
    });

    serde_json::to_string(&ChatMessage {
        sender: message.sender,
        text: message.text,
    })
    .expect("chat message should serialize")
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(message)) => {
                        if let Some(text) = message.as_text() {
                            let payload = normalize_payload(text, addr);
                            println!("Rust server received from {addr}: {payload}");
                            bcast_tx.send(payload)?;
                        }
                    }
                    Some(Err(err)) => return Err(Box::new(err) as Box<dyn Error + Send + Sync>),
                    None => break,
                }
            }
            broadcast = bcast_rx.recv() => {
                let payload = broadcast?;
                ws_stream.send(Message::text(payload)).await?;
            }
        }
    }

    println!("Rust server closed connection from {addr}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(32);
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Haekal Alexander Dinova Rust websocket server listening on ws://127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Rust server accepted YewChat client from {addr}");
        let bcast_tx = bcast_tx.clone();

        tokio::spawn(async move {
            let (_request, ws_stream) = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}
