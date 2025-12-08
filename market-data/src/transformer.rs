use crate::types::{DepthEvent, Event as WsEvent, TradeEvent, UserOrderUpdateEvent};
use protocol::types::{
    BookUpdate, Event as EngineEvent, Fill, OrderAck, OrderCancelled, OrderReject, Trade,
};

pub struct Transformer;

impl Transformer {
    pub fn new() -> Self {
        Self
    }

    pub fn transform(&self, event: EngineEvent) -> WsEvent {
        match event {
            // Public Events
            EngineEvent::Trade(trade) => self.transform_trade(trade),
            EngineEvent::BookUpdate(book_update) => self.transform_depth(book_update),

            // Private Events
            EngineEvent::Fill(fill) => self.transform_fill(fill),
            EngineEvent::OrderAck(order_ack) => self.transform_order_ack(order_ack),
            EngineEvent::OrderReject(order_reject) => self.transform_order_reject(order_reject),
            EngineEvent::OrderCancelled(order_cancelled) => {
                self.transform_order_cancelled(order_cancelled)
            }
        }
    }

    pub fn transform_trade(&self, trade: Trade) -> WsEvent {
        WsEvent::Trade(TradeEvent {
            trade_id: trade.trade_id,
            symbol: trade.symbol,
            price: trade.price,
            quantity: trade.quantity,
            timestamp: trade.timestamp,
        })
    }

    pub fn transform_depth(&self, book_update: BookUpdate) -> WsEvent {
        WsEvent::Depth(DepthEvent {
            symbol: book_update.symbol,
            bids: book_update.bids,
            asks: book_update.asks,
            last_price: book_update.last_price,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn transform_fill(&self, fill: Fill) -> WsEvent {
        WsEvent::OrderUpdate(UserOrderUpdateEvent::Fill {
            order_id: fill.order_id,
            user_id: fill.user_id,
            symbol: fill.symbol,
            filled_quantity: fill.filled_quantity,
            filled_price: fill.filled_price,
            remaining_quantity: fill.remaining_quantity,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn transform_order_ack(&self, order_ack: OrderAck) -> WsEvent {
        WsEvent::OrderUpdate(UserOrderUpdateEvent::Ack {
            order_id: order_ack.order_id,
            user_id: order_ack.user_id,
            symbol: order_ack.symbol,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn transform_order_reject(&self, order_reject: OrderReject) -> WsEvent {
        WsEvent::OrderUpdate(UserOrderUpdateEvent::Reject {
            order_id: order_reject.order_id,
            user_id: order_reject.user_id,
            reason: order_reject.reason.to_string(),
            message: order_reject.message,
            timestamp: chrono::Utc::now().timestamp_millis(),
            symbol: order_reject.symbol,
        })
    }

    pub fn transform_order_cancelled(&self, order_cancelled: OrderCancelled) -> WsEvent {
        WsEvent::OrderUpdate(UserOrderUpdateEvent::Cancelled {
            order_id: order_cancelled.order_id,
            user_id: order_cancelled.user_id,
            symbol: order_cancelled.symbol,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }
}
