use futures_util::{SinkExt, stream::SplitSink};
use std::collections::{HashMap, HashSet};
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

pub struct UserInfo {
    pub user_id: Option<u64>,
    pub writer: SplitSink<WebSocketStream<TcpStream>, Message>,
    pub subscribed_trades: HashSet<String>,
    pub subscribed_tickers: HashSet<String>,
    pub subscribed_depth: HashSet<String>,
}

pub struct UserManager {
    pub users: HashMap<String, UserInfo>,
    pub user_map: HashMap<u64, String>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            user_map: HashMap::new(),
        }
    }

    pub fn add_user(
        &mut self,
        user_addr: &str,
        writer: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) {
        self.users.insert(
            user_addr.to_string(),
            UserInfo {
                user_id: None,
                writer,
                subscribed_trades: HashSet::new(),
                subscribed_tickers: HashSet::new(),
                subscribed_depth: HashSet::new(),
            },
        );

        println!("[UserManager] New WS user added: {}", user_addr);
    }

    pub fn remove_user(&mut self, user_addr: &str) {
        if let Some(user) = self.users.remove(user_addr) {
            if let Some(uid) = user.user_id {
                self.user_map.remove(&uid);
                println!("[UserManager] WS user removed: {}", user_addr);
            } else {
                println!("[UserManager] WS user not associated: {}", user_addr);
            }
        } else {
            println!("[UserManager] WS user not found: {}", user_addr);
        }
    }

    pub fn associate_user(&mut self, user_addr: &str, user_id: u64) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.user_id = Some(user_id);
            self.user_map.insert(user_id, user_addr.to_string());
            println!(
                "[UserManager] User associated: {} -> {}",
                user_addr, user_id
            );
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }

    pub fn disassociate_user(&mut self, user_addr: &str) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.user_id = None;

            if let Some(uid) = user.user_id {
                self.user_map.remove(&uid);
                println!("[UserManager] User disassociated: {}", user_addr);
            } else {
                println!("[UserManager] User not associated: {}", user_addr);
            }
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }

    pub async fn send_order_update(&mut self, user_id: u64, order_update: &str) {
        if let Some(user_addr) = self.user_map.get(&user_id) {
            if let Some(user) = self.users.get_mut(user_addr) {
                let message = Message::text(order_update);
                if let Err(e) = user.writer.send(message).await {
                    eprintln!("Could not send order update, error occured: {}", e);
                }
            } else {
                println!("[UserManager] User not found: {}", user_addr);
            }
        } else {
            println!("[UserManager] User not found: {}", user_id);
        }
    }
}

impl UserManager {
    pub fn subscribe_trade(&mut self, user_addr: &str, symbol: &str) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.subscribed_trades.insert(symbol.to_string());
            println!(
                "[UserManager] User subscribed to trade: {} -> {}",
                user_addr, symbol
            );
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }

    pub fn unsubscribe_trade(&mut self, user_addr: &str, symbol: &str) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.subscribed_trades.remove(symbol);
            println!(
                "[UserManager] User unsubscribed from trade: {} -> {}",
                user_addr, symbol
            );
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }

    pub async fn broadcast_trade(&mut self, symbol: &str, trade: &str) {
        for user in self.users.values_mut() {
            if user.subscribed_trades.contains(symbol) {
                let message = Message::text(trade);
                if let Err(e) = user.writer.send(message).await {
                    eprintln!("Could not send trade, error occured: {}", e);
                }
            }
        }
    }
}

impl UserManager {
    pub fn subscribe_ticker(&mut self, user_addr: &str, symbol: &str) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.subscribed_tickers.insert(symbol.to_string());
            println!(
                "[UserManager] User subscribed to ticker: {} -> {}",
                user_addr, symbol
            );
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }

    pub fn unsubscribe_ticker(&mut self, user_addr: &str, symbol: &str) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.subscribed_tickers.remove(symbol);
            println!(
                "[UserManager] User unsubscribed from ticker: {} -> {}",
                user_addr, symbol
            );
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }

    pub async fn broadcast_ticker(&mut self, symbol: &str, ticker: &str) {
        for user in self.users.values_mut() {
            if user.subscribed_tickers.contains(symbol) {
                let message = Message::text(ticker);
                if let Err(e) = user.writer.send(message).await {
                    eprintln!("Could not send ticker, error occured: {}", e);
                }
            }
        }
    }
}

impl UserManager {
    pub fn subscribe_depth(&mut self, user_addr: &str, symbol: &str) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.subscribed_depth.insert(symbol.to_string());
            println!(
                "[UserManager] User subscribed to depth: {} -> {}",
                user_addr, symbol
            );
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }
}

impl UserManager {
    pub fn unsubscribe_depth(&mut self, user_addr: &str, symbol: &str) {
        if let Some(user) = self.users.get_mut(user_addr) {
            user.subscribed_depth.remove(symbol);
            println!(
                "[UserManager] User unsubscribed from depth: {} -> {}",
                user_addr, symbol
            );
        } else {
            println!("[UserManager] User not found: {}", user_addr);
        }
    }

    pub async fn broadcast_depth(&mut self, symbol: &str, depth: &str) {
        for user in self.users.values_mut() {
            if user.subscribed_depth.contains(symbol) {
                let message = Message::text(depth);
                if let Err(e) = user.writer.send(message).await {
                    eprintln!("Could not send depth, error occured: {}", e);
                }
            }
        }
    }
}
