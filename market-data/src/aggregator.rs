use crate::types::{DepthEvent, Event, TickerEvent, TradeEvent};
use chrono::Utc;
use std::collections::HashMap;

pub struct Aggregator {
    // Depth batching state per symbol
    last_depth: HashMap<String, DepthEvent>,
    last_depth_emit: HashMap<String, i64>,
    depth_interval_ms: i64,

    // Ticker state per symbol
    ticker_state: HashMap<String, TickerState>,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            last_depth: HashMap::new(),
            last_depth_emit: HashMap::new(),
            depth_interval_ms: 100, // 100 ms
            ticker_state: HashMap::new(),
        }
    }

    pub fn process(&mut self, ev: Event) -> Vec<Event> {
        let mut out = vec![];

        match &ev {
            Event::Trade(t) => {
                self.update_ticker_from_trade(t);
                out.push(ev.clone());

                if let Some(ticker) = self.build_ticker_event(t) {
                    out.push(Event::Ticker(ticker));
                }
            }

            Event::Depth(depth) => {
                let symbol = depth.symbol.clone();
                self.last_depth.insert(symbol.clone(), depth.clone());
                let now = Utc::now().timestamp_millis();

                let last_emit = self.last_depth_emit.get(&symbol).copied().unwrap_or(0);

                if now - last_emit >= self.depth_interval_ms {
                    self.last_depth_emit.insert(symbol.clone(), now);

                    if let Some(latest) = self.last_depth.get(&symbol) {
                        out.push(Event::Depth(latest.clone()));
                    }
                }
            }

            Event::OrderUpdate(_) => {
                out.push(ev);
            }

            Event::Ticker(_) => {
                out.push(ev);
            }
        }

        out
    }

    fn update_ticker_from_trade(&mut self, t: &TradeEvent) {
        let symbol = &t.symbol;
        let state = self
            .ticker_state
            .entry(symbol.clone())
            .or_insert_with(TickerState::new);

        let price = t.price;
        let qty = t.quantity;
        let ts = t.timestamp;

        // Initialize open/high/low with first trade
        if state.open_24h.is_none() {
            state.open_24h = Some(price);
            state.high_24h = price;
            state.low_24h = price;
        }

        state.last_price = Some(price);
        state.last_trade_time = ts;

        state.volume_24h = state.volume_24h.saturating_add(qty);

        if state.open_24h.is_none() {
            state.open_24h = Some(price);
        }

        if price > state.high_24h {
            state.high_24h = price;
        }
        if price < state.low_24h {
            state.low_24h = price;
        }
    }

    fn build_ticker_event(&self, t: &TradeEvent) -> Option<TickerEvent> {
        let state = self.ticker_state.get(&t.symbol)?;
        if state.last_price.is_none() || state.open_24h.is_none() {
            return None;
        }

        Some(TickerEvent {
            symbol: t.symbol.clone(),
            last_price: state.last_price.unwrap(),
            open: state.open_24h.unwrap(),
            high: state.high_24h,
            low: state.low_24h,
            volume: state.volume_24h,
            price_change: state.last_price.unwrap() as i64 - state.open_24h.unwrap() as i64,
            price_change_percent: ((state.last_price.unwrap() as i64
                - state.open_24h.unwrap() as i64) as f64
                / state.open_24h.unwrap() as f64)
                * 100.0,
            timestamp: Utc::now().timestamp_millis(),
        })
    }
}

#[derive(Debug, Clone)]
struct TickerState {
    last_price: Option<u64>,
    open_24h: Option<u64>,
    high_24h: u64,
    low_24h: u64,
    volume_24h: u64,
    last_trade_time: i64,
}

impl TickerState {
    fn new() -> Self {
        Self {
            last_price: None,
            open_24h: None,
            high_24h: 0,
            low_24h: u64::MAX,
            volume_24h: 0,
            last_trade_time: 0,
        }
    }
}
