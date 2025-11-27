use crossbeam_channel::{Receiver, Sender};
use protocol::{
    CancelOrder, CancelReason, Event, Order, OrderAck, OrderCancelled, OrderCommand, OrderReject,
    RejectReason,
};

/// synchronous matching engine
/// runs in a dedicated thread, no async, deteministic, locks free
///
/// The engine is responsible for:
/// - Receiving events from the client
/// - Matching orders
/// - Sending events to the client
/// - Handling errors
/// - Logging
/// - Metrics

pub struct Engine {
    // TODO: Add orderbook state and active_orders here later
}

impl Engine {
    pub fn new() -> Self {
        Self {
            // TODO: Initialize orderbook
        }
    }

    pub fn run(&mut self, order_rx: Receiver<OrderCommand>, event_tx: Sender<Event>) {
        println!("[Engine] Starting matching engine...");

        loop {
            match order_rx.recv() {
                Ok(OrderCommand::PlaceOrder(order)) => {
                    println!("[Engine] Placing order: {order:?}");
                    self.handle_place_order(order, &event_tx);
                }
                Ok(OrderCommand::CancelOrder(cancel_order)) => {
                    println!("[Engine] Cancelling order: {cancel_order:?}");
                    self.handle_cancel_order(cancel_order, &event_tx);
                }
                Err(e) => {
                    println!("[Engine] Error receiving order command: {e}");
                    break;
                }
            }
        }

        println!("[Engine] Engine shutting down");
    }

    fn handle_place_order(&self, order: Order, event_tx: &Sender<Event>) {
        println!(
            "[Engine] Processing order: {} from user {}",
            order.order_id, order.user_id
        );

        if order.quantity == 0 {
            let reject = Event::OrderReject(OrderReject {
                order_id: order.order_id,
                user_id: order.user_id,
                reason: RejectReason::InvalidQuantity,
            });

            if let Err(e) = event_tx.send(reject) {
                eprintln!("[Engine] Failed to send event: {}", e);
            };
            return;
        }

        let ack = Event::OrderAck(OrderAck {
            order_id: order.order_id,
            user_id: order.user_id,
            symbol: order.symbol,
        });

        if let Err(e) = event_tx.send(ack) {
            eprintln!("[Engine] Failed to send event: {}", e);
        }

        return;

        // TODO: match order in the orderbook later
    }

    fn handle_cancel_order(&self, cancel_order: CancelOrder, event_tx: &Sender<Event>) {
        println!(
            "[Engine] Cancelling order: {} from user {}",
            cancel_order.order_id, cancel_order.user_id
        );

        let cancelled = Event::OrderCancelled(OrderCancelled {
            order_id: cancel_order.order_id,
            user_id: cancel_order.user_id,
            symbol: cancel_order.symbol,
            reason: CancelReason::UserRequested,
        });

        if let Err(e) = event_tx.send(cancelled) {
            eprint!("[Engine] Failed to send event: {}", e);
        };

        return;

        // TODO: cancel order in the orderbook later
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
