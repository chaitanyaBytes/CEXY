use futures_util::StreamExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};

use crate::ws::{
    client_manager::UserManager,
    types::{Event, Method, WsClientMessage},
};

pub async fn handle_connection(
    stream: TcpStream,
    user_addr: String,
    user_manager: Arc<RwLock<UserManager>>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[ws] handshake failed from {}: {}", user_addr, e);
            return;
        }
    };

    println!("[ws] connection established from {}", user_addr);

    handle_stream(ws_stream, &user_addr, user_manager.clone()).await;
}

pub async fn handle_stream(
    ws_stream: WebSocketStream<TcpStream>,
    user_addr: &str,
    user_manager: Arc<RwLock<UserManager>>,
) {
    let (write, mut read) = ws_stream.split();

    {
        let mut manager = user_manager.write().await;
        manager.add_user(user_addr, write);
        println!("WebSocket connection established from: {}", user_addr);
    }

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("[ws] received message: {}", text);
                let parsed: Result<WsClientMessage, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(parsed) => {
                        handle_message(parsed, &user_addr, user_manager.clone()).await;
                    }
                    Err(e) => {
                        eprintln!("[ws] error parsing message: {}", e);
                    }
                }
            }

            Ok(Message::Binary(bin)) => {
                println!("[ws] received binary message: {}", bin.len());
            }

            Ok(Message::Ping(ping)) => {
                println!("[ws] received ping: {:?}", ping);
            }

            Ok(Message::Close(close)) => {
                println!("[ws] received close: {:?}", close);
                let mut manager = user_manager.write().await;
                manager.remove_user(user_addr);
                println!("WebSocket connection closed from: {}", user_addr);
            }

            Err(e) => {
                eprintln!("[ws] read error from {}: {}", user_addr, e);
            }

            _ => {}
        }
    }
}

async fn handle_message(
    msg: WsClientMessage,
    user_addr: &str,
    user_manager: Arc<RwLock<UserManager>>,
) {
    match msg.event {
        Event::TRADE => match msg.method {
            Method::SUBSCRIBE => {
                user_manager
                    .write()
                    .await
                    .subscribe_trade(&user_addr.to_string(), &msg.symbol);
            }
            Method::UNSUBSCRIBE => {
                user_manager
                    .write()
                    .await
                    .unsubscribe_trade(&user_addr.to_string(), &msg.symbol);
            }
        },
        Event::DEPTH => match msg.method {
            Method::SUBSCRIBE => {
                user_manager
                    .write()
                    .await
                    .subscribe_depth(&user_addr, &msg.symbol);
            }
            Method::UNSUBSCRIBE => {
                user_manager
                    .write()
                    .await
                    .unsubscribe_depth(&user_addr, &msg.symbol);
            }
        },
        Event::TICKER => match msg.method {
            Method::SUBSCRIBE => {
                user_manager
                    .write()
                    .await
                    .subscribe_ticker(&user_addr, &msg.symbol);
            }
            Method::UNSUBSCRIBE => {
                user_manager
                    .write()
                    .await
                    .unsubscribe_ticker(&user_addr, &msg.symbol);
            }
        },
        Event::ORDERUPDATE => match msg.method {
            Method::SUBSCRIBE => {
                user_manager
                    .write()
                    .await
                    .associate_user(user_addr, msg.user_id.unwrap());
            }
            Method::UNSUBSCRIBE => {
                user_manager.write().await.disassociate_user(user_addr);
            }
        },
    }
}
