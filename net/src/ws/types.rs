use enum_stringify::EnumStringify;
use serde::Deserialize;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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

#[derive(Deserialize, PartialEq, Eq, Hash, EnumIter, EnumStringify, Clone)]
pub enum RegisteredSymbols {
    SOL_USDC,
    BTC_USDc,
    ETH_USDC,
}

impl RegisteredSymbols {
    pub fn from_str(asset: &str) -> Option<Self> {
        for symbol in RegisteredSymbols::iter() {
            if symbol.to_string() == asset {
                return Some(symbol);
            }
        }
        None
    }
}
