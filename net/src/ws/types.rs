use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WsClientMessage {
    pub user_id: Option<u64>,
    pub method: Method,
    pub event: Event,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub enum Method {
    SUBSCRIBE,
    UNSUBSCRIBE,
}

#[derive(Debug, Deserialize, PartialEq, Hash, Clone)]
pub enum Event {
    TRADE,
    DEPTH,
    TICKER,
    ORDERUPDATE,
}
