use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};

use crate::ws::types::WsClientMessage;

pub struct WsServerApp {
    pub port: u16,
    pub handle: JoinHandle<()>,
}

impl WsServerApp {
    pub async fn build(host: &str, port: &str) -> Result<Self, std::io::Error> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
        let listener = TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");

        let port = listener.local_addr()?.port();

        println!("WebSocket server running on {}", addr);

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, user_addr)) => {
                        tokio::spawn(handle_connection(stream, user_addr));
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        continue;
                    }
                }
            }
        });

        Ok(Self { port, handle })
    }

    pub async fn run_until_stopped(self) -> anyhow::Result<()> {
        self.handle.await?;
        Ok(())
    }
}

async fn handle_connection(stream: TcpStream, user_addr: SocketAddr) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[ws] handshake failed from {}: {}", user_addr, e);
            return;
        }
    };

    println!("[ws] connection established from {}", user_addr);

    handle_stream(ws_stream, user_addr).await;
}

async fn handle_stream(ws_stream: WebSocketStream<TcpStream>, user_addr: SocketAddr) {
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("[ws] received message: {}", text);
                let parsed: Result<WsClientMessage, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(parsed) => {
                        println!("[ws] parsed message: {:?}", parsed);
                    }
                    Err(e) => {
                        eprintln!("[ws] error parsing message: {}", e);
                    }
                }

                let _ = write.send(Message::Text(text)).await;
            }
            Ok(Message::Binary(bin)) => {
                println!("[ws] received binary message: {}", bin.len());
            }
            Ok(Message::Ping(ping)) => {
                println!("[ws] received ping: {:?}", ping);
                let _ = write.send(Message::Pong(ping)).await;
            }
            Ok(Message::Close(close)) => {
                println!("[ws] received close: {:?}", close);
            }
            Err(e) => {
                eprintln!("[ws] read error from {}: {}", user_addr, e);
            }
            _ => {}
        }
    }
}
