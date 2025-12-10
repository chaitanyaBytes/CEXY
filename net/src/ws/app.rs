use redis::Client;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock, task::JoinHandle};

use crate::ws::broadcasters::{
    depth::broadcast_depth_events, order_update::broadcast_order_update_events,
    ticker::broadcast_ticker_events, trade::broadcast_trade_events,
};
use crate::ws::{client_manager::UserManager, lib::handle_connection};

pub struct WsServerApp {
    pub port: u16,
    pub handle: JoinHandle<()>,
    pub broadcaster_handles: Vec<JoinHandle<()>>,
}

impl WsServerApp {
    pub async fn build(host: &str, port: &str) -> Result<Self, std::io::Error> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
        let listener = TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");

        let port = listener.local_addr()?.port();

        println!("WebSocket server running on {}", addr);

        let user_manager = Arc::new(RwLock::new(UserManager::new()));

        let redis_url = "redis://127.0.0.1:6379";
        let redis_client = Client::open(redis_url).expect("[ws] unable to create redis client");

        let trade_user_manager = user_manager.clone();
        let depth_user_manager = user_manager.clone();
        let ticker_user_manager = user_manager.clone();
        let order_update_user_manager = user_manager.clone();

        let mut broadcaster_handles = Vec::new();

        let redis_trade = redis_client.clone();
        broadcaster_handles.push(tokio::spawn(async move {
            let _ = broadcast_trade_events(trade_user_manager, redis_trade).await;
        }));

        let redis_depth = redis_client.clone();
        broadcaster_handles.push(tokio::spawn(async move {
            let _ = broadcast_depth_events(depth_user_manager, redis_depth).await;
        }));

        let redis_ticker = redis_client.clone();
        broadcaster_handles.push(tokio::spawn(async move {
            let _ = broadcast_ticker_events(ticker_user_manager, redis_ticker).await;
        }));

        let redis_order = redis_client.clone();
        broadcaster_handles.push(tokio::spawn(async move {
            let _ = broadcast_order_update_events(order_update_user_manager, redis_order).await;
        }));

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, user_addr)) => {
                        tokio::spawn(handle_connection(
                            stream,
                            user_addr.to_string(),
                            user_manager.clone(),
                        ));
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                        continue;
                    }
                }
            }
        });

        Ok(Self {
            port,
            handle,
            broadcaster_handles,
        })
    }

    pub async fn run_until_stopped(self) -> anyhow::Result<()> {
        self.handle.await?;
        Ok(())
    }

    pub fn abort_broadcasters(&self) {
        for handle in &self.broadcaster_handles {
            handle.abort();
        }
    }
}
