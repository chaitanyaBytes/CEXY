# Cexy

A high-performance cryptocurrency exchange built in Rust, featuring a low-latency matching engine, real-time market data distribution, and scalable architecture.

## Overview

Cexy is a centralized exchange (CEX) implementation designed for high throughput and low latency. It implements a price-time priority matching engine with support for limit and market orders, real-time market data streaming via WebSockets, and persistent event storage.

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                         Client Layer                            │
│  ┌──────────────┐                    ┌──────────────────────┐   │
│  │ HTTP Clients │                    │ WebSocket Clients    │   │
│  └──────┬───────┘                    └──────────┬───────────┘   │
│         │                                       │               │
└─────────┼───────────────────────────────────────┼───────────────┘
          │                                       │
          ▼                                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Gateway                                 │
│  ┌──────────────┐                    ┌──────────────────────┐   │
│  │ HTTP Server  │                    │ WebSocket Server     │   │
│  │  (Port 8080) │                    │  (Port 8081)         │   │
│  └──────┬───────┘                    └──────────┬───────────┘   │
│         │                                       │               │
│         └──────────────┬────────────────────────┘               │
│                        │                                        │
│                        ▼                                        │
│              ┌──────────────────┐                               │
│              │  Order Channel   │                               │
│              └────────┬─────────┘                               │
└───────────────────────┼─────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Matching Engine                              │
│  ┌──────────────────────────────────────────────────────────┐   │ 
│  │  OrderBook (Price-Time Priority)                         │   │
│  │  - BTreeMap<Price, VecDeque<Order>> (Bids/Asks)          │   │
│  │  - FIFO within price levels                              │   │
│  │  - Immediate matching on order placement                 │   │
│  └───────────────────────┬──────────────────────────────────┘   │
│                          │                                      │
│                          ▼                                      │
│              ┌──────────────────────┐                           │
│              │   Event Channel      │                           │
│              │  (Unbounded)         │                           │
│              └───────────┬──────────┘                           │
└──────────────────────────┼──────────────────────────────────────┘
                           │
                           ▼
        ┌──────────────────┴──────────────────┐
        │                                     │
        ▼                                     ▼
┌───────────────┐                    ┌──────────────────────┐
│ Event         │                    │  Persistence Writer │
│ Broadcaster   │                    │  (ScyllaDB)         │
└───────┬───────┘                    └──────────────────────┘
        │
        ├──────────────────┬──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────────┐  ┌──────────────┐
│ Market Data  │  │  Redis Pub/Sub   │  │  WebSocket   │
│ Pipeline     │  │  (Scalable)      │  │  Broadcasters│
│              │  └──────────────────┘  └──────────────┘
│ - Transform  │
│ - Aggregate  │
│ - Publish    │
└──────────────┘ 
```

### Component Details

#### Gateway (`gateway/`)

The main orchestrator that coordinates all system components:

- Manages HTTP and WebSocket servers
- Routes orders to the matching engine
- Distributes events to market data and persistence layers
- Handles graceful shutdown

#### Engine Core (`engine-core/`)

The synchronous matching engine running on a dedicated thread:

- **OrderBook**: Price-time priority order book using `BTreeMap` for price levels and `VecDeque` for FIFO ordering
- **Matching Logic**: Immediate matching on order placement
- **Event Generation**: Emits events for all order lifecycle changes (ack, fill, trade, cancel, reject)

#### Market Data (`market-data/`)

Processes engine events and prepares them for distribution:

- **Transformer**: Converts internal engine events to WebSocket-friendly format
- **Aggregator**: Calculates tickers (24h stats), throttles depth updates, manages per-symbol state
- **Publisher**: Publishes events to Redis Pub/Sub for scalable distribution

#### Network Layer (`net/`)

Provides HTTP REST API and WebSocket streaming:

- **HTTP Server**: REST endpoints for order placement, cancellation, and depth queries
- **WebSocket Server**: Real-time streaming of trades, depth updates, tickers, and user order updates
- **Client Manager**: Manages WebSocket connections, subscriptions, and user associations

#### Persistence (`persistence/`)

Writes all events to ScyllaDB for audit and recovery:

- **Batch Writes**: Groups events into batches (100 events or 100ms timeout) for optimal throughput
- **Schema**: Tables for orders, trades, cancel_orders, markets, tickers
- **Order State Tracking**: Maintains order state for accurate persistence

#### Protocol (`protocol/`)

Shared data structures and types:

- Order commands and responses
- Event types
- Serialization/deserialization

## Features

### Order Matching

- **Price-Time Priority**: Orders at the same price are matched in FIFO order
- **Immediate Matching**: Orders are matched immediately upon placement
- **Partial Fills**: Orders can be partially filled
- **Order Types**: Limit and Market orders
- **Order Sides**: Buy and Sell

### Market Data

- **Real-time Trades**: Trade events streamed via WebSocket
- **Orderbook Depth**: Throttled depth updates (configurable per symbol)
- **Tickers**: 24-hour statistics (open, high, low, last price, volume, price change)
- **User Order Updates**: Private order status updates for authenticated users

### Scalability

- **Horizontal Scaling**: WebSocket servers can scale independently via Redis Pub/Sub
- **Event Broadcasting**: Single engine broadcasts to multiple consumers (market data, persistence)
- **Connection Pooling**: Redis connection pooling for efficient resource usage
- **Batch Processing**: Persistence layer uses batch writes for optimal database throughput

### Performance

- **Lock-free ID Generation**: AtomicU64 for order ID generation
- **Dedicated Threads**: Engine runs on dedicated thread for predictable latency
- **Bounded Channels**: Backpressure handling for persistence layer

## Getting Started

### Prerequisites

- Rust 1.70+ (edition 2024)
- Docker (for Redis and ScyllaDB)
- Docker Compose (optional, for local development)

### Running the Exchange

1. **Start dependencies**:

   ```bash
   # Start Redis
   docker run -d -p 6379:6379 redis:latest
   
   # Start ScyllaDB
   docker run -d -p 9042:9042 scylladb/scylla:latest
   ```

2. **Build and run the gateway**:

   ```bash
   cargo build --release
   cargo run --bin gateway
   ```

3. **Access the services**:
   - HTTP API: `http://127.0.0.1:8080`
   - WebSocket: `ws://127.0.0.1:8081`

### API Endpoints

#### Place Order

```bash
POST /api/v1/orders/open
Content-Type: application/json

{
  "user_id": 100,
  "symbol": "SOL_USDC",
  "side": "Buy",
  "order_type": "Limit",
  "quantity": 50,
  "price": 50000
}
```

#### Cancel Order

```bash
DELETE /api/v1/orders/cancel
Content-Type: application/json

{
  "user_id": 100,
  "symbol": "SOL_USDC",
  "order_id": 12345
}
```

#### Get Depth

```bash
GET /api/v1/depth/{symbol}?limit=20
```

### WebSocket Protocol

#### Subscribe to Trades

```json
{
  "method": "subscribe",
  "params": {
    "channels": ["trade:SOL_USDC"]
  }
}
```

#### Subscribe to Depth

```json
{
  "method": "subscribe",
  "params": {
    "channels": ["depth:SOL_USDC"]
  }
}
```

#### Subscribe to Ticker

```json
{
  "method": "subscribe",
  "params": {
    "channels": ["ticker:SOL_USDC"]
  }
}
```

## Project Structure

```
cexy/
├── gateway/          # Main application orchestrator
├── engine-core/      # Matching engine and orderbook
├── market-data/      # Event processing and distribution
├── net/              # HTTP and WebSocket servers
├── persistence/      # ScyllaDB event persistence
├── protocol/         # Shared types and data structures
└── runtime/          # Tokio runtime wrapper
```

## Testing

Run all tests:

```bash
cargo test
```

Run tests for a specific crate:

```bash
cargo test -p engine-core
cargo test -p market-data
cargo test -p protocol
cargo test -p net
```

## Configuration

### Environment Variables

- `REDIS_URL`: Redis connection string (default: `redis://127.0.0.1:6379`)
- `SCYLLA_HOST`: ScyllaDB hostname (default: `127.0.0.1`)
- `SCYLLA_KEYSPACE`: ScyllaDB keyspace name (default: `orderbook`)
- `HTTP_PORT`: HTTP server port (default: `8080`)
- `WS_PORT`: WebSocket server port (default: `8081`)

### Performance Tuning

- **Order Channel Size**: Adjust bounded channel size in `gateway/src/main.rs` (default: 1000)
- **Persistence Batch Size**: Modify `BATCH_SIZE` in `persistence/src/writer.rs` (default: 100)
- **Persistence Batch Timeout**: Modify `BATCH_TIMEOUT_MS` in `persistence/src/writer.rs` (default: 100ms)
- **Redis Pool Size**: Configure in `RedisPublisher::new()` (default: 10)

## Design Decisions

### Why Synchronous Engine?

The matching engine runs on a dedicated thread to ensure:

- Predictable latency (no async overhead)
- Lock-free order book operations
- Deterministic matching behavior

### Why Redis Pub/Sub?

Redis Pub/Sub enables:

- Horizontal scaling of WebSocket servers
- Decoupling of event producers and consumers
- Simple fan-out pattern for market data

### Why ScyllaDB?

ScyllaDB provides:

- High write throughput for event streaming
- Time-series optimized data model
- Horizontal scalability
- Low latency for reads

## Future Enhancements

- [ ] Multi-symbol support (currently single symbol: SOL_USDC)
- [ ] Order recovery from persistence on startup
- [ ] Authentication and authorization
- [ ] Rate limiting
- [ ] Order history API
- [ ] Admin endpoints
- [ ] Metrics and monitoring
- [ ] Circuit breakers for external dependencies
- [ ] Order validation enhancements
- [ ] Stop-loss and take-profit orders

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
