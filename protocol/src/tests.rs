use crate::types::*;
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    // Order Tests

    #[test]
    fn test_order_new() {
        let order = Order::new(
            1,
            100,
            "SOL_USDC".to_string(),
            Side::Buy,
            OrderType::Limit,
            50,
            Some(50000),
        );

        assert_eq!(order.order_id, 1);
        assert_eq!(order.user_id, 100);
        assert_eq!(order.symbol, "SOL_USDC");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.quantity, 50);
        assert_eq!(order.price, Some(50000));
    }

    #[test]
    fn test_order_market_order() {
        let order = Order::new(
            1,
            100,
            "SOL_USDC".to_string(),
            Side::Buy,
            OrderType::Market,
            50,
            None, // Market orders don't have price
        );

        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.price, None);
    }

    #[test]
    fn test_order_serialization() {
        let order = Order {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 50,
            price: Some(50000),
        };

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();

        assert_eq!(order.order_id, deserialized.order_id);
        assert_eq!(order.user_id, deserialized.user_id);
        assert_eq!(order.symbol, deserialized.symbol);
        assert_eq!(order.side, deserialized.side);
        assert_eq!(order.order_type, deserialized.order_type);
        assert_eq!(order.quantity, deserialized.quantity);
        assert_eq!(order.price, deserialized.price);
    }

    #[test]
    fn test_order_deserialization_with_null_price() {
        // Market order JSON with null price
        let json = r#"{
            "order_id": 1,
            "user_id": 100,
            "symbol": "SOL_USDC",
            "side": "Buy",
            "order_type": "Market",
            "quantity": 50,
            "price": null
        }"#;

        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.price, None);
    }

    // ========== CancelOrder Tests ==========

    #[test]
    fn test_cancel_order_new() {
        let cancel = CancelOrder::new(1, 100, "SOL_USDC".to_string());

        assert_eq!(cancel.order_id, 1);
        assert_eq!(cancel.user_id, 100);
        assert_eq!(cancel.symbol, "SOL_USDC");
    }

    #[test]
    fn test_cancel_order_serialization() {
        let cancel = CancelOrder {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };

        let json = serde_json::to_string(&cancel).unwrap();
        let deserialized: CancelOrder = serde_json::from_str(&json).unwrap();

        assert_eq!(cancel.order_id, deserialized.order_id);
        assert_eq!(cancel.user_id, deserialized.user_id);
        assert_eq!(cancel.symbol, deserialized.symbol);
    }

    // OrderCommand Tests

    #[test]
    fn test_order_command_place_order_serialization() {
        let order = Order {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 50,
            price: Some(50000),
        };

        let command = OrderCommand::PlaceOrder(order);
        let json = serde_json::to_string(&command).unwrap();
        let deserialized: OrderCommand = serde_json::from_str(&json).unwrap();

        match (command, deserialized) {
            (OrderCommand::PlaceOrder(o1), OrderCommand::PlaceOrder(o2)) => {
                assert_eq!(o1.order_id, o2.order_id);
                assert_eq!(o1.user_id, o2.user_id);
                assert_eq!(o1.symbol, o2.symbol);
            }
            _ => panic!("Commands don't match"),
        }
    }

    #[test]
    fn test_order_command_cancel_order_serialization() {
        let cancel = CancelOrder {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };

        let command = OrderCommand::CancelOrder(cancel);
        let json = serde_json::to_string(&command).unwrap();
        let deserialized: OrderCommand = serde_json::from_str(&json).unwrap();

        match (command, deserialized) {
            (OrderCommand::CancelOrder(c1), OrderCommand::CancelOrder(c2)) => {
                assert_eq!(c1.order_id, c2.order_id);
                assert_eq!(c1.user_id, c2.user_id);
                assert_eq!(c1.symbol, c2.symbol);
            }
            _ => panic!("Commands don't match"),
        }
    }

    #[test]
    fn test_order_command_get_depth_serialization() {
        let command = OrderCommand::GetDepth;
        let json = serde_json::to_string(&command).unwrap();
        let deserialized: OrderCommand = serde_json::from_str(&json).unwrap();

        match (command, deserialized) {
            (OrderCommand::GetDepth, OrderCommand::GetDepth) => {}
            _ => panic!("Commands don't match"),
        }
    }

    // Event Tests

    #[test]
    fn test_event_trade_serialization() {
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
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match (event, deserialized) {
            (Event::Trade(t1), Event::Trade(t2)) => {
                assert_eq!(t1.trade_id, t2.trade_id);
                assert_eq!(t1.price, t2.price);
                assert_eq!(t1.quantity, t2.quantity);
                assert_eq!(t1.symbol, t2.symbol);
            }
            _ => panic!("Events don't match"),
        }
    }

    #[test]
    fn test_event_fill_serialization() {
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
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match (event, deserialized) {
            (Event::Fill(f1), Event::Fill(f2)) => {
                assert_eq!(f1.order_id, f2.order_id);
                assert_eq!(f1.user_id, f2.user_id);
                assert_eq!(f1.filled_quantity, f2.filled_quantity);
                assert_eq!(f1.filled_price, f2.filled_price);
            }
            _ => panic!("Events don't match"),
        }
    }

    #[test]
    fn test_event_order_ack_serialization() {
        let ack = OrderAck {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };

        let event = Event::OrderAck(ack);
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match (event, deserialized) {
            (Event::OrderAck(a1), Event::OrderAck(a2)) => {
                assert_eq!(a1.order_id, a2.order_id);
                assert_eq!(a1.user_id, a2.user_id);
                assert_eq!(a1.symbol, a2.symbol);
            }
            _ => panic!("Events don't match"),
        }
    }

    #[test]
    fn test_event_order_reject_serialization() {
        let reject = OrderReject {
            order_id: 1,
            user_id: 100,
            reason: RejectReason::InvalidQuantity,
            message: "Quantity must be greater than 0".to_string(),
            symbol: "SOL_USDC".to_string(),
        };

        let event = Event::OrderReject(reject);
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match (event, deserialized) {
            (Event::OrderReject(r1), Event::OrderReject(r2)) => {
                assert_eq!(r1.order_id, r2.order_id);
                assert_eq!(r1.user_id, r2.user_id);
                assert_eq!(r1.reason.to_string(), r2.reason.to_string());
                assert_eq!(r1.message, r2.message);
            }
            _ => panic!("Events don't match"),
        }
    }

    #[test]
    fn test_event_order_cancelled_serialization() {
        let cancelled = OrderCancelled {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            reason: CancelReason::UserRequested,
        };

        let event = Event::OrderCancelled(cancelled);
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match (event, deserialized) {
            (Event::OrderCancelled(c1), Event::OrderCancelled(c2)) => {
                assert_eq!(c1.order_id, c2.order_id);
                assert_eq!(c1.user_id, c2.user_id);
                assert_eq!(c1.symbol, c2.symbol);
            }
            _ => panic!("Events don't match"),
        }
    }

    #[test]
    fn test_event_book_update_serialization() {
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
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match (event, deserialized) {
            (Event::BookUpdate(b1), Event::BookUpdate(b2)) => {
                assert_eq!(b1.symbol, b2.symbol);
                assert_eq!(b1.bids.len(), b2.bids.len());
                assert_eq!(b1.asks.len(), b2.asks.len());
                assert_eq!(b1.last_price, b2.last_price);
            }
            _ => panic!("Events don't match"),
        }
    }

    // Side Tests

    #[test]
    fn test_side_equality() {
        assert_eq!(Side::Buy, Side::Buy);
        assert_eq!(Side::Sell, Side::Sell);
        assert_ne!(Side::Buy, Side::Sell);
    }

    #[test]
    fn test_side_serialization() {
        let buy = Side::Buy;
        let sell = Side::Sell;

        let buy_json = serde_json::to_string(&buy).unwrap();
        let sell_json = serde_json::to_string(&sell).unwrap();

        assert_eq!(buy_json, "\"Buy\"");
        assert_eq!(sell_json, "\"Sell\"");

        let buy_deserialized: Side = serde_json::from_str(&buy_json).unwrap();
        let sell_deserialized: Side = serde_json::from_str(&sell_json).unwrap();

        assert_eq!(buy, buy_deserialized);
        assert_eq!(sell, sell_deserialized);
    }

    // OrderType Tests

    #[test]
    fn test_order_type_equality() {
        assert_eq!(OrderType::Limit, OrderType::Limit);
        assert_eq!(OrderType::Market, OrderType::Market);
        assert_ne!(OrderType::Limit, OrderType::Market);
    }

    #[test]
    fn test_order_type_serialization() {
        let limit = OrderType::Limit;
        let market = OrderType::Market;

        let limit_json = serde_json::to_string(&limit).unwrap();
        let market_json = serde_json::to_string(&market).unwrap();

        assert_eq!(limit_json, "\"Limit\"");
        assert_eq!(market_json, "\"Market\"");

        let limit_deserialized: OrderType = serde_json::from_str(&limit_json).unwrap();
        let market_deserialized: OrderType = serde_json::from_str(&market_json).unwrap();

        assert_eq!(limit, limit_deserialized);
        assert_eq!(market, market_deserialized);
    }

    // RejectReason Tests

    #[test]
    fn test_reject_reason_to_string() {
        let reasons = vec![
            (RejectReason::InvalidPrice, "InvalidPrice"),
            (RejectReason::InvalidOrder, "InvalidOrder"),
            (RejectReason::InvalidQuantity, "InvalidQuantity"),
            (RejectReason::InsufficientBalance, "InsufficientBalance"),
            (RejectReason::SymbolNotFound, "SymbolNotFound"),
            (RejectReason::MarketClosed, "MarketClosed"),
            (RejectReason::InternalError, "InternalError"),
        ];

        for (reason, expected) in reasons {
            assert_eq!(reason.to_string(), expected);
        }
    }

    #[test]
    fn test_reject_reason_serialization() {
        let reason = RejectReason::InvalidQuantity;
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: RejectReason = serde_json::from_str(&json).unwrap();

        assert_eq!(reason.to_string(), deserialized.to_string());
    }

    // CancelReason Tests

    #[test]
    fn test_cancel_reason_serialization() {
        let reasons = vec![
            CancelReason::UserRequested,
            CancelReason::SystemCancelled,
            CancelReason::Expired,
            CancelReason::Liquidation,
        ];

        for reason in reasons {
            let json = serde_json::to_string(&reason).unwrap();
            let _deserialized: CancelReason = serde_json::from_str(&json).unwrap();
            // Can't directly compare, but should deserialize correctly
        }
    }

    // OrderStatus Tests

    #[test]
    fn test_order_status_serialization() {
        let statuses = vec![
            OrderStatus::Pending,
            OrderStatus::PartiallyFilled,
            OrderStatus::Filled,
            OrderStatus::Cancelled,
            OrderStatus::Rejected,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let _deserialized: OrderStatus = serde_json::from_str(&json).unwrap();
            // Can't directly compare, but should deserialize correctly
        }
    }

    // PriceLevel Tests

    #[test]
    fn test_price_level_serialization() {
        let price_level = PriceLevel {
            price: 50000,
            quantity: 100,
        };

        let json = serde_json::to_string(&price_level).unwrap();
        let deserialized: PriceLevel = serde_json::from_str(&json).unwrap();

        assert_eq!(price_level.price, deserialized.price);
        assert_eq!(price_level.quantity, deserialized.quantity);
    }

    // Round-trip Tests

    #[test]
    fn test_complete_order_round_trip() {
        let original = Order {
            order_id: 12345,
            user_id: 67890,
            symbol: "BTC/USD".to_string(),
            side: Side::Sell,
            order_type: OrderType::Market,
            quantity: 999,
            price: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();

        assert_eq!(original.order_id, deserialized.order_id);
        assert_eq!(original.user_id, deserialized.user_id);
        assert_eq!(original.symbol, deserialized.symbol);
        assert_eq!(original.side, deserialized.side);
        assert_eq!(original.order_type, deserialized.order_type);
        assert_eq!(original.quantity, deserialized.quantity);
        assert_eq!(original.price, deserialized.price);
    }

    #[test]
    fn test_complete_trade_round_trip() {
        let original = Trade {
            trade_id: 999,
            maker_order_id: 100,
            maker_user_id: 200,
            taker_order_id: 300,
            taker_user_id: 400,
            symbol: "ETH/USD".to_string(),
            quantity: 500,
            price: 60000,
            timestamp: 1234567890123,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Trade = serde_json::from_str(&json).unwrap();

        assert_eq!(original.trade_id, deserialized.trade_id);
        assert_eq!(original.maker_order_id, deserialized.maker_order_id);
        assert_eq!(original.taker_order_id, deserialized.taker_order_id);
        assert_eq!(original.quantity, deserialized.quantity);
        assert_eq!(original.price, deserialized.price);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_complete_book_update_round_trip() {
        let original = BookUpdate {
            symbol: "SOL_USDC".to_string(),
            bids: vec![
                PriceLevel {
                    price: 49900,
                    quantity: 100,
                },
                PriceLevel {
                    price: 49800,
                    quantity: 200,
                },
            ],
            asks: vec![
                PriceLevel {
                    price: 50100,
                    quantity: 50,
                },
                PriceLevel {
                    price: 50200,
                    quantity: 75,
                },
            ],
            last_price: Some(50000),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: BookUpdate = serde_json::from_str(&json).unwrap();

        assert_eq!(original.symbol, deserialized.symbol);
        assert_eq!(original.bids.len(), deserialized.bids.len());
        assert_eq!(original.asks.len(), deserialized.asks.len());
        assert_eq!(original.last_price, deserialized.last_price);

        for (b1, b2) in original.bids.iter().zip(deserialized.bids.iter()) {
            assert_eq!(b1.price, b2.price);
            assert_eq!(b1.quantity, b2.quantity);
        }
    }

    // Edge Cases

    #[test]
    fn test_order_with_zero_quantity() {
        let order = Order {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 0,
            price: Some(50000),
        };

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.quantity, 0);
    }

    #[test]
    fn test_order_with_max_values() {
        let order = Order {
            order_id: u64::MAX,
            user_id: u64::MAX,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: u64::MAX,
            price: Some(u64::MAX),
        };

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.order_id, u64::MAX);
        assert_eq!(deserialized.user_id, u64::MAX);
        assert_eq!(deserialized.quantity, u64::MAX);
        assert_eq!(deserialized.price, Some(u64::MAX));
    }

    #[test]
    fn test_empty_book_update() {
        let book_update = BookUpdate {
            symbol: "SOL_USDC".to_string(),
            bids: vec![],
            asks: vec![],
            last_price: None,
        };

        let json = serde_json::to_string(&book_update).unwrap();
        let deserialized: BookUpdate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.bids.len(), 0);
        assert_eq!(deserialized.asks.len(), 0);
        assert_eq!(deserialized.last_price, None);
    }

    #[test]
    fn test_large_book_update() {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for i in 0..100 {
            bids.push(PriceLevel {
                price: 50000 - i,
                quantity: 100 + i,
            });
            asks.push(PriceLevel {
                price: 50000 + i,
                quantity: 100 + i,
            });
        }

        let book_update = BookUpdate {
            symbol: "SOL_USDC".to_string(),
            bids,
            asks,
            last_price: Some(50000),
        };

        let json = serde_json::to_string(&book_update).unwrap();
        let deserialized: BookUpdate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.bids.len(), 100);
        assert_eq!(deserialized.asks.len(), 100);
    }
}
