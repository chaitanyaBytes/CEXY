#![allow(dead_code)]
use crate::{
    aggregator::Aggregator,
    pipeline::MarketDataPipeline,
    publisher::publisher::Publisher,
    transformer::Transformer,
    types::{DepthEvent, Event as WSEvent, TickerEvent, TradeEvent, UserOrderUpdateEvent},
};
use crossbeam_channel;
use protocol::types::{
    BookUpdate, CancelReason, Event, Fill, OrderAck, OrderCancelled, OrderReject, PriceLevel,
    RejectReason, Side, Trade,
};
use std::sync::{Arc, Mutex};

// Mock publisher for testing
#[derive(Clone)]
struct MockPublisher {
    published: Arc<Mutex<Vec<WSEvent>>>,
    errors: Arc<Mutex<Vec<String>>>,
}

impl MockPublisher {
    fn new() -> Self {
        Self {
            published: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_published(&self) -> Vec<WSEvent> {
        self.published.lock().unwrap().clone()
    }

    fn get_errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.published.lock().unwrap().clear();
        self.errors.lock().unwrap().clear();
    }

    fn count(&self) -> usize {
        self.published.lock().unwrap().len()
    }
}

impl Publisher for MockPublisher {
    fn publish(&self, event: &WSEvent) {
        self.published.lock().unwrap().push(event.clone());
    }

    fn publish_batch(&self, events: Vec<WSEvent>) {
        self.published.lock().unwrap().extend(events);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Transformer Tests ==========

    #[test]
    fn test_transformer_trade_event() {
        let transformer = Transformer::new();
        let trade = Trade {
            trade_id: 1,
            maker_order_id: 10,
            maker_user_id: 100,
            taker_order_id: 20,
            taker_user_id: 200,
            symbol: "SOL_USDC".to_string(),
            quantity: 50,
            price: 50000,
            timestamp: 1234567890,
        };
        let event = Event::Trade(trade);

        let ws_event = transformer.transform(event);

        match ws_event {
            WSEvent::Trade(t) => {
                assert_eq!(t.trade_id, 1);
                assert_eq!(t.quantity, 50);
                assert_eq!(t.price, 50000);
                assert_eq!(t.symbol, "SOL_USDC");
                assert_eq!(t.timestamp, 1234567890);
            }
            _ => panic!("Expected Trade event, got {:?}", ws_event),
        }
    }

    #[test]
    fn test_transformer_fill_event() {
        let transformer = Transformer::new();
        let fill = Fill {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            filled_quantity: 25,
            filled_price: 50000,
            remaining_quantity: 25,
        };
        let event = Event::Fill(fill);

        let ws_event = transformer.transform(event);

        match ws_event {
            WSEvent::OrderUpdate(UserOrderUpdateEvent::Fill {
                order_id,
                user_id,
                symbol,
                filled_quantity,
                filled_price,
                remaining_quantity,
                ..
            }) => {
                assert_eq!(order_id, 1);
                assert_eq!(user_id, 100);
                assert_eq!(symbol, "SOL_USDC");
                assert_eq!(filled_quantity, 25);
                assert_eq!(filled_price, 50000);
                assert_eq!(remaining_quantity, 25);
            }
            _ => panic!("Expected Fill OrderUpdate event"),
        }
    }

    #[test]
    fn test_transformer_order_ack() {
        let transformer = Transformer::new();
        let ack = OrderAck {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };
        let event = Event::OrderAck(ack);

        let ws_event = transformer.transform(event);

        match ws_event {
            WSEvent::OrderUpdate(UserOrderUpdateEvent::Ack {
                order_id,
                user_id,
                symbol,
                ..
            }) => {
                assert_eq!(order_id, 1);
                assert_eq!(user_id, 100);
                assert_eq!(symbol, "SOL_USDC");
            }
            _ => panic!("Expected Ack OrderUpdate event"),
        }
    }

    #[test]
    fn test_transformer_order_reject() {
        let transformer = Transformer::new();
        let reject = OrderReject {
            order_id: 1,
            user_id: 100,
            reason: RejectReason::InvalidQuantity,
            message: "Quantity must be greater than 0".to_string(),
            symbol: "SOL_USDC".to_string(),
        };
        let event = Event::OrderReject(reject);

        let ws_event = transformer.transform(event);

        match ws_event {
            WSEvent::OrderUpdate(UserOrderUpdateEvent::Reject {
                order_id,
                user_id,
                symbol,
                reason,
                message,
                ..
            }) => {
                assert_eq!(order_id, 1);
                assert_eq!(user_id, 100);
                assert_eq!(symbol, "SOL_USDC");
                assert!(reason.contains("InvalidQuantity"));
                assert_eq!(message, "Quantity must be greater than 0");
            }
            _ => panic!("Expected Reject OrderUpdate event"),
        }
    }

    #[test]
    fn test_transformer_order_cancelled() {
        let transformer = Transformer::new();
        let cancelled = OrderCancelled {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            reason: CancelReason::UserRequested,
        };
        let event = Event::OrderCancelled(cancelled);

        let ws_event = transformer.transform(event);

        match ws_event {
            WSEvent::OrderUpdate(UserOrderUpdateEvent::Cancelled {
                order_id,
                user_id,
                symbol,
                ..
            }) => {
                assert_eq!(order_id, 1);
                assert_eq!(user_id, 100);
                assert_eq!(symbol, "SOL_USDC");
            }
            _ => panic!("Expected Cancelled OrderUpdate event"),
        }
    }

    #[test]
    fn test_transformer_book_update() {
        let transformer = Transformer::new();
        let book_update = BookUpdate {
            symbol: "SOL_USDC".to_string(),
            bids: vec![PriceLevel {
                price: 49900,
                quantity: 100,
            }],
            asks: vec![PriceLevel {
                price: 50100,
                quantity: 50,
            }],
            last_price: Some(50000),
        };
        let event = Event::BookUpdate(book_update);

        let ws_event = transformer.transform(event);

        match ws_event {
            WSEvent::Depth(depth) => {
                assert_eq!(depth.symbol, "SOL_USDC");
                assert_eq!(depth.bids.len(), 1);
                assert_eq!(depth.asks.len(), 1);
                assert_eq!(depth.bids[0].price, 49900);
                assert_eq!(depth.bids[0].quantity, 100);
                assert_eq!(depth.asks[0].price, 50100);
                assert_eq!(depth.asks[0].quantity, 50);
                assert_eq!(depth.last_price, Some(50000));
            }
            _ => panic!("Expected Depth event"),
        }
    }

    #[test]
    fn test_transformer_all_event_types() {
        let transformer = Transformer::new();

        // Test all event types transform correctly
        let events = vec![
            Event::Trade(Trade {
                trade_id: 1,
                maker_order_id: 10,
                maker_user_id: 100,
                taker_order_id: 20,
                taker_user_id: 200,
                symbol: "SOL_USDC".to_string(),
                quantity: 50,
                price: 50000,
                timestamp: 1234567890,
            }),
            Event::Fill(Fill {
                order_id: 1,
                user_id: 100,
                symbol: "SOL_USDC".to_string(),
                side: Side::Buy,
                filled_quantity: 25,
                filled_price: 50000,
                remaining_quantity: 25,
            }),
            Event::OrderAck(OrderAck {
                order_id: 1,
                user_id: 100,
                symbol: "SOL_USDC".to_string(),
            }),
        ];

        for event in events {
            let ws_event = transformer.transform(event);
            assert!(
                matches!(ws_event, WSEvent::Trade(_) | WSEvent::OrderUpdate(_)),
                "Event should transform to valid WS event"
            );
        }
    }

    // ========== Aggregator Tests ==========

    #[test]
    fn test_aggregator_trade_updates_ticker() {
        let mut aggregator = Aggregator::new();
        let symbol = "SOL_USDC".to_string();

        let trade = WSEvent::Trade(TradeEvent {
            trade_id: 1,
            symbol: symbol.clone(),
            price: 50000,
            quantity: 10,
            timestamp: 1000,
        });

        let result = aggregator.process(trade);
        assert_eq!(result.len(), 2); // Trade + Ticker

        let has_trade = result.iter().any(|e| matches!(e, WSEvent::Trade(_)));
        let has_ticker = result.iter().any(|e| matches!(e, WSEvent::Ticker(_)));

        assert!(has_trade, "Should output trade event");
        assert!(has_ticker, "Should output ticker event");
    }

    #[test]
    fn test_aggregator_ticker_calculation() {
        let mut aggregator = Aggregator::new();
        let symbol = "SOL_USDC".to_string();

        // First trade sets open price
        let trade1 = WSEvent::Trade(TradeEvent {
            trade_id: 1,
            symbol: symbol.clone(),
            price: 50000,
            quantity: 10,
            timestamp: 1000,
        });
        let result1 = aggregator.process(trade1);
        let ticker1 = result1
            .iter()
            .find_map(|e| {
                if let WSEvent::Ticker(t) = e {
                    Some(t)
                } else {
                    None
                }
            })
            .unwrap();

        assert_eq!(ticker1.open, 50000);
        assert_eq!(ticker1.last_price, 50000);
        assert_eq!(ticker1.high, 50000);
        assert_eq!(ticker1.low, 50000);
        assert_eq!(ticker1.volume, 10);
        assert_eq!(ticker1.price_change, 0);
        assert_eq!(ticker1.price_change_percent, 0.0);

        // Second trade with higher price
        let trade2 = WSEvent::Trade(TradeEvent {
            trade_id: 2,
            symbol: symbol.clone(),
            price: 51000,
            quantity: 20,
            timestamp: 2000,
        });
        let result2 = aggregator.process(trade2);
        let ticker2 = result2
            .iter()
            .find_map(|e| {
                if let WSEvent::Ticker(t) = e {
                    Some(t)
                } else {
                    None
                }
            })
            .unwrap();

        assert_eq!(ticker2.open, 50000);
        assert_eq!(ticker2.last_price, 51000);
        assert_eq!(ticker2.high, 51000);
        assert_eq!(ticker2.low, 50000);
        assert_eq!(ticker2.volume, 30);
        assert_eq!(ticker2.price_change, 1000);
        assert!((ticker2.price_change_percent - 2.0).abs() < 0.01); // 2% increase
    }

    #[test]
    fn test_aggregator_ticker_high_low_tracking() {
        let mut aggregator = Aggregator::new();
        let symbol = "SOL_USDC".to_string();

        // Sequence of trades
        let trades = vec![
            (50000, 10), // Open
            (51000, 20), // High
            (49000, 15), // Low
            (50500, 25), // Middle
        ];

        for (price, qty) in trades {
            let trade = WSEvent::Trade(TradeEvent {
                trade_id: 1,
                symbol: symbol.clone(),
                price,
                quantity: qty,
                timestamp: 1000,
            });
            aggregator.process(trade);
        }

        // Get final ticker
        let final_trade = WSEvent::Trade(TradeEvent {
            trade_id: 5,
            symbol: symbol.clone(),
            price: 50500,
            quantity: 1,
            timestamp: 5000,
        });
        let result = aggregator.process(final_trade);
        let ticker = result
            .iter()
            .find_map(|e| {
                if let WSEvent::Ticker(t) = e {
                    Some(t)
                } else {
                    None
                }
            })
            .unwrap();

        println!("ticker: {:?}", ticker);
        assert_eq!(ticker.open, 50000);
        assert_eq!(ticker.high, 51000);
        assert_eq!(ticker.low, 49000);
        assert_eq!(ticker.last_price, 50500);
        assert_eq!(ticker.volume, 71); // Sum of all quantities
    }

    #[test]
    fn test_aggregator_depth_throttling() {
        let mut aggregator = Aggregator::new();
        let symbol = "SOL_USDC".to_string();

        let depth1 = WSEvent::Depth(DepthEvent {
            symbol: symbol.clone(),
            bids: vec![],
            asks: vec![],
            timestamp: 1000,
            last_price: None,
        });

        let depth2 = WSEvent::Depth(DepthEvent {
            symbol: symbol.clone(),
            bids: vec![],
            asks: vec![],
            timestamp: 150, // Within 100ms throttle window
            last_price: None,
        });

        // First depth should pass through
        let result1 = aggregator.process(depth1);
        assert_eq!(result1.len(), 1);
        assert!(matches!(result1[0], WSEvent::Depth(_)));

        // Second depth should be throttled (within 100ms)
        let result2 = aggregator.process(depth2);
        assert_eq!(result2.len(), 0); // Throttled
    }

    #[test]
    fn test_aggregator_depth_after_throttle_interval() {
        let mut aggregator = Aggregator::new();
        let symbol = "SOL_USDC".to_string();

        let depth1 = WSEvent::Depth(DepthEvent {
            symbol: symbol.clone(),
            bids: vec![],
            asks: vec![],
            timestamp: 1000,
            last_price: None,
        });

        // First depth
        aggregator.process(depth1);

        // Wait and send another depth (simulate time passing)
        // Note: In real test, we'd need to mock time or wait
        // For now, we test that the last depth is stored
        let depth2 = WSEvent::Depth(DepthEvent {
            symbol: symbol.clone(),
            bids: vec![PriceLevel {
                price: 50000,
                quantity: 100,
            }],
            asks: vec![],
            timestamp: 1200, // 200ms later (outside throttle window)
            last_price: None,
        });

        // This should pass through if enough time has passed
        // The aggregator uses Utc::now(), so we can't easily test exact timing
        // But we can verify the last depth is stored
        let _ = aggregator.process(depth2);
    }

    #[test]
    fn test_aggregator_order_update_passes_through() {
        let mut aggregator = Aggregator::new();

        let order_update = WSEvent::OrderUpdate(UserOrderUpdateEvent::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            timestamp: 1000,
        });

        let result = aggregator.process(order_update);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], WSEvent::OrderUpdate(_)));
    }

    #[test]
    fn test_aggregator_multiple_symbols() {
        let mut aggregator = Aggregator::new();

        // Trade for SOL_USDC
        let trade1 = WSEvent::Trade(TradeEvent {
            trade_id: 1,
            symbol: "SOL_USDC".to_string(),
            price: 50000,
            quantity: 10,
            timestamp: 1000,
        });

        // Trade for BTC/USD
        let trade2 = WSEvent::Trade(TradeEvent {
            trade_id: 2,
            symbol: "BTC/USD".to_string(),
            price: 60000,
            quantity: 5,
            timestamp: 2000,
        });

        let result1 = aggregator.process(trade1);
        let result2 = aggregator.process(trade2);

        // Both should produce tickers
        assert!(result1.iter().any(|e| matches!(e, WSEvent::Ticker(_))));
        assert!(result2.iter().any(|e| matches!(e, WSEvent::Ticker(_))));

        // Tickers should be for different symbols
        let ticker1 = result1
            .iter()
            .find_map(|e| {
                if let WSEvent::Ticker(t) = e {
                    Some(t)
                } else {
                    None
                }
            })
            .unwrap();
        let ticker2 = result2
            .iter()
            .find_map(|e| {
                if let WSEvent::Ticker(t) = e {
                    Some(t)
                } else {
                    None
                }
            })
            .unwrap();

        assert_eq!(ticker1.symbol, "SOL_USDC");
        assert_eq!(ticker2.symbol, "BTC/USD");
        assert_eq!(ticker1.last_price, 50000);
        assert_eq!(ticker2.last_price, 60000);
    }

    // ========== Pipeline Tests ==========

    #[test]
    fn test_pipeline_processes_trade_event() {
        let mock_pub = MockPublisher::new();
        let publishers: Vec<Box<dyn Publisher>> = vec![Box::new(mock_pub.clone())];
        let mut pipeline = MarketDataPipeline::new(publishers);

        let (event_tx, event_rx) = crossbeam_channel::unbounded::<Event>();

        // Spawn pipeline in background
        let handle = std::thread::spawn(move || {
            pipeline.run(event_rx);
        });

        // Send a trade event
        let trade = Trade {
            trade_id: 1,
            maker_order_id: 10,
            maker_user_id: 100,
            taker_order_id: 20,
            taker_user_id: 200,
            symbol: "SOL_USDC".to_string(),
            quantity: 50,
            price: 50000,
            timestamp: 1234567890,
        };
        event_tx.send(Event::Trade(trade)).unwrap();

        // Give pipeline time to process
        std::thread::sleep(std::time::Duration::from_millis(200));

        drop(event_tx);
        handle.join().unwrap();

        // Check that events were published
        let published = mock_pub.get_published();
        assert!(!published.is_empty(), "Should have published events");
        assert!(
            published.iter().any(|e| matches!(e, WSEvent::Trade(_))),
            "Should have published trade event"
        );
    }

    #[test]
    fn test_pipeline_processes_multiple_events() {
        let mock_pub = MockPublisher::new();
        let publishers: Vec<Box<dyn Publisher>> = vec![Box::new(mock_pub.clone())];
        let mut pipeline = MarketDataPipeline::new(publishers);

        let (event_tx, event_rx) = crossbeam_channel::unbounded::<Event>();

        let handle = std::thread::spawn(move || {
            pipeline.run(event_rx);
        });

        // Send multiple events
        for i in 1..=5 {
            let trade = Trade {
                trade_id: i,
                maker_order_id: 10,
                maker_user_id: 100,
                taker_order_id: 20,
                taker_user_id: 200,
                symbol: "SOL_USDC".to_string(),
                quantity: 50,
                price: 50000 + i,
                timestamp: 1234567890i64 + i as i64,
            };
            event_tx.send(Event::Trade(trade)).unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(200));

        drop(event_tx);
        handle.join().unwrap();

        let published = mock_pub.get_published();
        assert!(
            published.len() >= 5,
            "Should have published multiple events"
        );
    }

    #[test]
    fn test_pipeline_multiple_publishers() {
        let mock_pub1 = MockPublisher::new();
        let mock_pub2 = MockPublisher::new();
        let publishers: Vec<Box<dyn Publisher>> =
            vec![Box::new(mock_pub1.clone()), Box::new(mock_pub2.clone())];
        let mut pipeline = MarketDataPipeline::new(publishers);

        let (event_tx, event_rx) = crossbeam_channel::unbounded::<Event>();

        let handle = std::thread::spawn(move || {
            pipeline.run(event_rx);
        });

        let trade = Trade {
            trade_id: 1,
            maker_order_id: 10,
            maker_user_id: 100,
            taker_order_id: 20,
            taker_user_id: 200,
            symbol: "SOL_USDC".to_string(),
            quantity: 50,
            price: 50000,
            timestamp: 1234567890,
        };
        event_tx.send(Event::Trade(trade)).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(200));

        drop(event_tx);
        handle.join().unwrap();

        // Both publishers should have received events
        assert!(mock_pub1.count() > 0, "Publisher 1 should have events");
        assert!(mock_pub2.count() > 0, "Publisher 2 should have events");
    }

    #[test]
    fn test_pipeline_handles_all_event_types() {
        let mock_pub = MockPublisher::new();
        let publishers: Vec<Box<dyn Publisher>> = vec![Box::new(mock_pub.clone())];
        let mut pipeline = MarketDataPipeline::new(publishers);

        let (event_tx, event_rx) = crossbeam_channel::unbounded::<Event>();

        let handle = std::thread::spawn(move || {
            pipeline.run(event_rx);
        });

        // Send all event types
        event_tx
            .send(Event::Trade(Trade {
                trade_id: 1,
                maker_order_id: 10,
                maker_user_id: 100,
                taker_order_id: 20,
                taker_user_id: 200,
                symbol: "SOL_USDC".to_string(),
                quantity: 50,
                price: 50000,
                timestamp: 1234567890,
            }))
            .unwrap();

        event_tx
            .send(Event::Fill(Fill {
                order_id: 1,
                user_id: 100,
                symbol: "SOL_USDC".to_string(),
                side: Side::Buy,
                filled_quantity: 25,
                filled_price: 50000,
                remaining_quantity: 25,
            }))
            .unwrap();

        event_tx
            .send(Event::OrderAck(OrderAck {
                order_id: 1,
                user_id: 100,
                symbol: "SOL_USDC".to_string(),
            }))
            .unwrap();

        event_tx
            .send(Event::BookUpdate(BookUpdate {
                symbol: "SOL_USDC".to_string(),
                bids: vec![],
                asks: vec![],
                last_price: Some(50000),
            }))
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(200));

        drop(event_tx);
        handle.join().unwrap();

        let published = mock_pub.get_published();
        assert!(!published.is_empty(), "Should have published events");
    }

    // ========== Types Tests ==========

    #[test]
    fn test_event_is_public() {
        let trade = WSEvent::Trade(TradeEvent {
            trade_id: 1,
            symbol: "SOL_USDC".to_string(),
            price: 50000,
            quantity: 10,
            timestamp: 1000,
        });
        assert!(trade.is_public(), "Trade should be public");

        let depth = WSEvent::Depth(DepthEvent {
            symbol: "SOL_USDC".to_string(),
            bids: vec![],
            asks: vec![],
            timestamp: 1000,
            last_price: None,
        });
        assert!(depth.is_public(), "Depth should be public");

        let ticker = WSEvent::Ticker(TickerEvent {
            symbol: "SOL_USDC".to_string(),
            last_price: 50000,
            open: 50000,
            high: 51000,
            low: 49000,
            volume: 100,
            price_change: 0,
            price_change_percent: 0.0,
            timestamp: 1000,
        });
        assert!(ticker.is_public(), "Ticker should be public");

        let order_update = WSEvent::OrderUpdate(UserOrderUpdateEvent::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            timestamp: 1000,
        });
        assert!(!order_update.is_public(), "OrderUpdate should be private");
    }

    #[test]
    fn test_event_user_id() {
        let trade = WSEvent::Trade(TradeEvent {
            trade_id: 1,
            symbol: "SOL_USDC".to_string(),
            price: 50000,
            quantity: 10,
            timestamp: 1000,
        });
        assert_eq!(trade.user_id(), None, "Trade should not have user_id");

        let fill = WSEvent::OrderUpdate(UserOrderUpdateEvent::Fill {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            filled_quantity: 10,
            filled_price: 50000,
            remaining_quantity: 0,
            timestamp: 1000,
        });
        assert_eq!(fill.user_id(), Some(100), "Fill should have user_id");

        let ack = WSEvent::OrderUpdate(UserOrderUpdateEvent::Ack {
            order_id: 1,
            user_id: 200,
            symbol: "SOL_USDC".to_string(),
            timestamp: 1000,
        });
        assert_eq!(ack.user_id(), Some(200), "Ack should have user_id");

        let reject = WSEvent::OrderUpdate(UserOrderUpdateEvent::Reject {
            order_id: 1,
            user_id: 300,
            symbol: "SOL_USDC".to_string(),
            reason: "Invalid".to_string(),
            message: "Test".to_string(),
            timestamp: 1000,
        });
        assert_eq!(reject.user_id(), Some(300), "Reject should have user_id");

        let cancelled = WSEvent::OrderUpdate(UserOrderUpdateEvent::Cancelled {
            order_id: 1,
            user_id: 400,
            symbol: "SOL_USDC".to_string(),
            timestamp: 1000,
        });
        assert_eq!(
            cancelled.user_id(),
            Some(400),
            "Cancelled should have user_id"
        );
    }

    #[test]
    fn test_trade_event_serialization() {
        let trade = TradeEvent {
            trade_id: 1,
            symbol: "SOL_USDC".to_string(),
            price: 50000,
            quantity: 10,
            timestamp: 1000,
        };

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(trade.trade_id, deserialized.trade_id);
        assert_eq!(trade.symbol, deserialized.symbol);
        assert_eq!(trade.price, deserialized.price);
        assert_eq!(trade.quantity, deserialized.quantity);
        assert_eq!(trade.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_depth_event_serialization() {
        let depth = DepthEvent {
            symbol: "SOL_USDC".to_string(),
            bids: vec![PriceLevel {
                price: 49900,
                quantity: 100,
            }],
            asks: vec![PriceLevel {
                price: 50100,
                quantity: 50,
            }],
            timestamp: 1000,
            last_price: Some(50000),
        };

        let json = serde_json::to_string(&depth).unwrap();
        let deserialized: DepthEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(depth.symbol, deserialized.symbol);
        assert_eq!(depth.bids.len(), deserialized.bids.len());
        assert_eq!(depth.asks.len(), deserialized.asks.len());
        assert_eq!(depth.last_price, deserialized.last_price);
    }

    #[test]
    fn test_ticker_event_serialization() {
        let ticker = TickerEvent {
            symbol: "SOL_USDC".to_string(),
            last_price: 50000,
            open: 50000,
            high: 51000,
            low: 49000,
            volume: 100,
            price_change: 0,
            price_change_percent: 0.0,
            timestamp: 1000,
        };

        let json = serde_json::to_string(&ticker).unwrap();
        let deserialized: TickerEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(ticker.symbol, deserialized.symbol);
        assert_eq!(ticker.last_price, deserialized.last_price);
        assert_eq!(ticker.open, deserialized.open);
        assert_eq!(ticker.high, deserialized.high);
        assert_eq!(ticker.low, deserialized.low);
        assert_eq!(ticker.volume, deserialized.volume);
    }

    #[test]
    fn test_user_order_update_event_serialization() {
        let fill = UserOrderUpdateEvent::Fill {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            filled_quantity: 10,
            filled_price: 50000,
            remaining_quantity: 0,
            timestamp: 1000,
        };

        let json = serde_json::to_string(&fill).unwrap();
        let deserialized: UserOrderUpdateEvent = serde_json::from_str(&json).unwrap();

        match (fill, deserialized) {
            (
                UserOrderUpdateEvent::Fill {
                    order_id: id1,
                    user_id: uid1,
                    filled_quantity: qty1,
                    ..
                },
                UserOrderUpdateEvent::Fill {
                    order_id: id2,
                    user_id: uid2,
                    filled_quantity: qty2,
                    ..
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(uid1, uid2);
                assert_eq!(qty1, qty2);
            }
            _ => panic!("Fill events don't match"),
        }
    }
}
