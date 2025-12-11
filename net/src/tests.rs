use crate::http::models::orders::*;
use protocol::types::*;
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    // OrderRequest Tests

    #[test]
    fn test_order_request_serialization() {
        let request = OrderRequest {
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 50,
            price: Some(50000),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.user_id, deserialized.user_id);
        assert_eq!(request.symbol, deserialized.symbol);
        assert_eq!(request.side, deserialized.side);
        assert_eq!(request.order_type, deserialized.order_type);
        assert_eq!(request.quantity, deserialized.quantity);
        assert_eq!(request.price, deserialized.price);
    }

    #[test]
    fn test_order_request_market_order() {
        let request = OrderRequest {
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Market,
            quantity: 50,
            price: None, // Market orders don't have price
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.order_type, OrderType::Market);
        assert_eq!(deserialized.price, None);
    }

    #[test]
    fn test_order_request_limit_order() {
        let request = OrderRequest {
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Sell,
            order_type: OrderType::Limit,
            quantity: 100,
            price: Some(60000),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.order_type, OrderType::Limit);
        assert_eq!(deserialized.price, Some(60000));
        assert_eq!(deserialized.side, Side::Sell);
    }

    // OrderResponse Tests

    #[test]
    fn test_order_response_ack_serialization() {
        let ack = OrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };

        let json = serde_json::to_string(&ack).unwrap();
        let deserialized: OrderResponse = serde_json::from_str(&json).unwrap();

        match (ack, deserialized) {
            (
                OrderResponse::Ack {
                    order_id: id1,
                    user_id: uid1,
                    symbol: s1,
                },
                OrderResponse::Ack {
                    order_id: id2,
                    user_id: uid2,
                    symbol: s2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(uid1, uid2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Responses don't match"),
        }
    }

    #[test]
    fn test_order_response_reject_serialization() {
        let reject = OrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidQuantity,
            symbol: "SOL_USDC".to_string(),
            message: "Quantity must be greater than 0".to_string(),
        };

        let json = serde_json::to_string(&reject).unwrap();
        let deserialized: OrderResponse = serde_json::from_str(&json).unwrap();

        match (reject, deserialized) {
            (
                OrderResponse::Reject {
                    order_id: id1,
                    reason: r1,
                    symbol: s1,
                    message: m1,
                },
                OrderResponse::Reject {
                    order_id: id2,
                    reason: r2,
                    symbol: s2,
                    message: m2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(r1.to_string(), r2.to_string());
                assert_eq!(s1, s2);
                assert_eq!(m1, m2);
            }
            _ => panic!("Responses don't match"),
        }
    }

    #[test]
    fn test_order_response_into_http_response() {
        let ack = OrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };

        let http_resp = ack.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 200);

        let reject = OrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidQuantity,
            symbol: "SOL_USDC".to_string(),
            message: "Test".to_string(),
        };

        let http_resp = reject.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 400);
    }

    // CancelOrderRequest Tests

    #[test]
    fn test_cancel_order_request_serialization() {
        let request = CancelOrderRequest {
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            order_id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CancelOrderRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.user_id, deserialized.user_id);
        assert_eq!(request.symbol, deserialized.symbol);
        assert_eq!(request.order_id, deserialized.order_id);
    }

    // CancelOrderResponse Tests

    #[test]
    fn test_cancel_order_response_ack_serialization() {
        let ack = CancelOrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };

        let json = serde_json::to_string(&ack).unwrap();
        let deserialized: CancelOrderResponse = serde_json::from_str(&json).unwrap();

        match (ack, deserialized) {
            (
                CancelOrderResponse::Ack {
                    order_id: id1,
                    user_id: uid1,
                    symbol: s1,
                },
                CancelOrderResponse::Ack {
                    order_id: id2,
                    user_id: uid2,
                    symbol: s2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(uid1, uid2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Responses don't match"),
        }
    }

    #[test]
    fn test_cancel_order_response_reject_serialization() {
        let reject = CancelOrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidOrder,
            message: "Order not found".to_string(),
        };

        let json = serde_json::to_string(&reject).unwrap();
        let deserialized: CancelOrderResponse = serde_json::from_str(&json).unwrap();

        match (reject, deserialized) {
            (
                CancelOrderResponse::Reject {
                    order_id: id1,
                    reason: r1,
                    message: m1,
                },
                CancelOrderResponse::Reject {
                    order_id: id2,
                    reason: r2,
                    message: m2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(r1.to_string(), r2.to_string());
                assert_eq!(m1, m2);
            }
            _ => panic!("Responses don't match"),
        }
    }

    #[test]
    fn test_cancel_order_response_into_http_response() {
        let ack = CancelOrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        };

        let http_resp = ack.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 200);

        let reject = CancelOrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidOrder,
            message: "Test".to_string(),
        };

        let http_resp = reject.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 400);
    }

    // CommandResponse Tests

    #[test]
    fn test_command_response_place_order_ack() {
        let place_order = CommandResponse::PlaceOrder(OrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        });

        match place_order {
            CommandResponse::PlaceOrder(OrderResponse::Ack { order_id, .. }) => {
                assert_eq!(order_id, 1);
            }
            _ => panic!("Expected PlaceOrder variant"),
        }
    }

    #[test]
    fn test_command_response_place_order_reject() {
        let place_order = CommandResponse::PlaceOrder(OrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidQuantity,
            symbol: "SOL_USDC".to_string(),
            message: "Test".to_string(),
        });

        match place_order {
            CommandResponse::PlaceOrder(OrderResponse::Reject { order_id, .. }) => {
                assert_eq!(order_id, 1);
            }
            _ => panic!("Expected PlaceOrder Reject variant"),
        }
    }

    #[test]
    fn test_command_response_cancel_order_ack() {
        let cancel_order = CommandResponse::CancelOrder(CancelOrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        });

        match cancel_order {
            CommandResponse::CancelOrder(CancelOrderResponse::Ack { order_id, .. }) => {
                assert_eq!(order_id, 1);
            }
            _ => panic!("Expected CancelOrder variant"),
        }
    }

    #[test]
    fn test_command_response_cancel_order_reject() {
        let cancel_order = CommandResponse::CancelOrder(CancelOrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidOrder,
            message: "Test".to_string(),
        });

        match cancel_order {
            CommandResponse::CancelOrder(CancelOrderResponse::Reject { order_id, .. }) => {
                assert_eq!(order_id, 1);
            }
            _ => panic!("Expected CancelOrder Reject variant"),
        }
    }

    #[test]
    fn test_command_response_depth() {
        let depth = CommandResponse::Depth(DepthResponse {
            bids: vec![(49900, 100), (49800, 200)],
            asks: vec![(50100, 50), (50200, 75)],
        });

        match depth {
            CommandResponse::Depth(d) => {
                assert_eq!(d.bids.len(), 2);
                assert_eq!(d.asks.len(), 2);
                assert_eq!(d.bids[0], (49900, 100));
                assert_eq!(d.asks[0], (50100, 50));
            }
            _ => panic!("Expected Depth variant"),
        }
    }

    #[test]
    fn test_command_response_into_http_response() {
        // Test PlaceOrder Ack
        let place_ack = CommandResponse::PlaceOrder(OrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        });
        let http_resp = place_ack.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 200);

        // Test PlaceOrder Reject
        let place_reject = CommandResponse::PlaceOrder(OrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidQuantity,
            symbol: "SOL_USDC".to_string(),
            message: "Test".to_string(),
        });
        let http_resp = place_reject.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 400);

        // Test CancelOrder Ack
        let cancel_ack = CommandResponse::CancelOrder(CancelOrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        });
        let http_resp = cancel_ack.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 200);

        // Test CancelOrder Reject
        let cancel_reject = CommandResponse::CancelOrder(CancelOrderResponse::Reject {
            order_id: 1,
            reason: RejectReason::InvalidOrder,
            message: "Test".to_string(),
        });
        let http_resp = cancel_reject.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 400);

        // Test Depth
        let depth = CommandResponse::Depth(DepthResponse {
            bids: vec![],
            asks: vec![],
        });
        let http_resp = depth.into_http_response();
        assert_eq!(http_resp.status().as_u16(), 200);
    }

    #[test]
    fn test_command_response_serialization() {
        let place_order = CommandResponse::PlaceOrder(OrderResponse::Ack {
            order_id: 1,
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
        });

        let json = serde_json::to_string(&place_order).unwrap();
        let deserialized: CommandResponse = serde_json::from_str(&json).unwrap();

        match (place_order, deserialized) {
            (
                CommandResponse::PlaceOrder(OrderResponse::Ack { order_id: id1, .. }),
                CommandResponse::PlaceOrder(OrderResponse::Ack { order_id: id2, .. }),
            ) => {
                assert_eq!(id1, id2);
            }
            _ => panic!("Responses don't match"),
        }
    }

    // DepthQuery Tests

    #[test]
    fn test_depth_query_serialization() {
        let query = DepthQuery { limit: 20 };

        let json = serde_json::to_string(&query).unwrap();
        let deserialized: DepthQuery = serde_json::from_str(&json).unwrap();

        assert_eq!(query.limit, deserialized.limit);
    }

    #[test]
    fn test_depth_query_different_limits() {
        for limit in [10, 20, 50, 100] {
            let query = DepthQuery { limit };
            let json = serde_json::to_string(&query).unwrap();
            let deserialized: DepthQuery = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.limit, limit);
        }
    }

    // DepthResponse Tests

    #[test]
    fn test_depth_response_serialization() {
        let response = DepthResponse {
            bids: vec![(49900, 100), (49800, 200)],
            asks: vec![(50100, 50), (50200, 75)],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: DepthResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.bids.len(), deserialized.bids.len());
        assert_eq!(response.asks.len(), deserialized.asks.len());
        assert_eq!(response.bids, deserialized.bids);
        assert_eq!(response.asks, deserialized.asks);
    }

    #[test]
    fn test_depth_response_empty() {
        let response = DepthResponse {
            bids: vec![],
            asks: vec![],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: DepthResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.bids.len(), 0);
        assert_eq!(deserialized.asks.len(), 0);
    }

    #[test]
    fn test_depth_response_large() {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for i in 0..100 {
            bids.push((50000 - i, 100 + i));
            asks.push((50000 + i, 100 + i));
        }

        let response = DepthResponse { bids, asks };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: DepthResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.bids.len(), 100);
        assert_eq!(deserialized.asks.len(), 100);
    }

    // Round-trip Tests

    #[test]
    fn test_complete_order_request_round_trip() {
        let original = OrderRequest {
            user_id: 12345,
            symbol: "BTC/USD".to_string(),
            side: Side::Sell,
            order_type: OrderType::Market,
            quantity: 999,
            price: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(original.user_id, deserialized.user_id);
        assert_eq!(original.symbol, deserialized.symbol);
        assert_eq!(original.side, deserialized.side);
        assert_eq!(original.order_type, deserialized.order_type);
        assert_eq!(original.quantity, deserialized.quantity);
        assert_eq!(original.price, deserialized.price);
    }

    #[test]
    fn test_complete_order_response_round_trip() {
        let original = OrderResponse::Ack {
            order_id: 999,
            user_id: 888,
            symbol: "ETH/USD".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: OrderResponse = serde_json::from_str(&json).unwrap();

        match (original, deserialized) {
            (
                OrderResponse::Ack {
                    order_id: id1,
                    user_id: uid1,
                    symbol: s1,
                },
                OrderResponse::Ack {
                    order_id: id2,
                    user_id: uid2,
                    symbol: s2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(uid1, uid2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Responses don't match"),
        }
    }

    // Edge Cases

    #[test]
    fn test_order_request_with_zero_quantity() {
        let request = OrderRequest {
            user_id: 100,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 0,
            price: Some(50000),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.quantity, 0);
    }

    #[test]
    fn test_order_request_with_max_values() {
        let request = OrderRequest {
            user_id: u64::MAX,
            symbol: "SOL_USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: u64::MAX,
            price: Some(u64::MAX),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user_id, u64::MAX);
        assert_eq!(deserialized.quantity, u64::MAX);
        assert_eq!(deserialized.price, Some(u64::MAX));
    }

    #[test]
    fn test_all_reject_reasons_in_response() {
        let reasons = vec![
            RejectReason::InvalidPrice,
            RejectReason::InvalidOrder,
            RejectReason::InvalidQuantity,
            RejectReason::InsufficientBalance,
            RejectReason::SymbolNotFound,
            RejectReason::MarketClosed,
            RejectReason::InternalError,
        ];

        for reason in reasons {
            let reject = OrderResponse::Reject {
                order_id: 1,
                reason: reason.clone(),
                symbol: "SOL_USDC".to_string(),
                message: "Test".to_string(),
            };

            let json = serde_json::to_string(&reject).unwrap();
            let deserialized: OrderResponse = serde_json::from_str(&json).unwrap();

            match deserialized {
                OrderResponse::Reject { reason: r, .. } => {
                    assert_eq!(r.to_string(), reason.to_string());
                }
                _ => panic!("Expected Reject"),
            }
        }
    }

    #[test]
    fn test_empty_symbol_string() {
        let request = OrderRequest {
            user_id: 100,
            symbol: "".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 50,
            price: Some(50000),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "");
    }

    #[test]
    fn test_unicode_symbol() {
        let request = OrderRequest {
            user_id: 100,
            symbol: "BTC/₿".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 50,
            price: Some(50000),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OrderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.symbol, "BTC/₿");
    }
}
