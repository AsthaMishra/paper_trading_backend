# Paper Trading Backend

A high-performance paper trading platform backend built with Rust, Axum, and ScyllaDB. Simulates cryptocurrency trading with live prices sourced from the Binance API.

## Pending / Roadmap

- **Error enums** — replace `Box<dyn Error>` in services with a typed `AppError` enum (`thiserror`). Implement `IntoResponse` so all routes return consistent JSON error bodies with correct HTTP status codes instead of ad-hoc `(StatusCode, String)` tuples.
- **Auth layer** — JWT middleware (`jsonwebtoken`). Add `POST /auth/login { wallet_address }` that returns a signed token. Apply `Authorization: Bearer` validation middleware to all protected routes. Store secret in `JWT_SECRET` env var.
- **Candlestick chart endpoint** — `GET /charts/:asset?interval=1h&limit=100` that proxies Binance klines (`/api/v3/klines`) and returns `[{ time, open, high, low, close, volume }]`. Pair with a `lightweight-charts` (TradingView) component on the frontend.
- **Input validation** — centralise asset whitelist (`SOL`, `BTC`, `ETH`, `BNB`) and reject unknown symbols before hitting the market data service.
- **Rate limiting** — add `tower_governor` or a simple per-IP counter to prevent abuse on trade and order endpoints.
- **Refresh token** — complement the JWT with a long-lived refresh token so the frontend can silently renew sessions without re-login.

## Features

- Live price feeds via Binance API with in-memory caching
- Buy/sell execution with 0.1% fee simulation
- Limit orders, stop-loss, and take-profit automation
- Portfolio tracking with unrealized and realized PnL
- Performance history snapshots on every trade
- Global leaderboard by realized PnL
- Paginated trade history and closed positions

## Tech Stack

| Layer | Technology |
|---|---|
| Web Framework | Axum 0.8 |
| Database | ScyllaDB (Cassandra-compatible) |
| Async Runtime | Tokio |
| Price Data | Binance REST API |
| HTTP Client | Reqwest |
| Serialization | Serde / Serde JSON |

## Prerequisites

- Rust (edition 2024)
- ScyllaDB running on `172.18.0.2:9042` (configurable via `.env`)
- Network access to `api.binance.com`

## Getting Started

```bash
# Start ScyllaDB (Docker)
docker run -d --name scylla -p 9042:9042 scylladb/scylla

# Run the server
RUST_LOG=info cargo run
```

The server starts on `0.0.0.0:8080` by default. Migrations run automatically on startup — no manual schema setup needed.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `APP_HOST` | `0.0.0.0` | Bind address |
| `APP_PORT` | `8080` | Bind port |
| `DB_HOSTS` | `172.18.0.2` | Comma-separated ScyllaDB hosts |
| `DB_PORT` | `9042` | ScyllaDB port |
| `DB_USERNAME` | `cassandra` | Database username |
| `DB_PASSWORD` | `cassandra` | Database password |
| `RUST_LOG` | `info` | Log level |

Copy `.env.example` to `.env` and adjust as needed.

## API Reference

### Health

```
GET /health → "OK"
```

---

### Users

```
POST /users
Body: { "wallet_address": "string" }
Response: { "message": "User created successfully" }
```
> Uses `INSERT IF NOT EXISTS` — safe to call for existing wallets.

```
GET /users/:wallet_address
Response: { "user": { ...User } | null }
```

**User object:**
```json
{
  "wallet_address": "string",
  "starting_balance": 10000.0,
  "current_balance": 10000.0,
  "total_realized_pnl": 0.0,
  "created_at": 1700000000000,
  "total_trades": 0,
  "winning_trades": 0,
  "best_trade": 0.0,
  "worst_trade": 0.0
}
```

---

### Trading

```
POST /trade
Body: {
  "wallet_address": "string",
  "asset": "SOL" | "BTC" | "ETH" | "BNB",
  "side": "buy" | "sell",
  "quantity": 1.0,
  "stop_loss": 100.0,    // optional
  "take_profit": 200.0   // optional
}
Response: { "message": "buy executed successfully" }
```
> Price is fetched live from Binance — no client-side price required.

```
GET /trade/:wallet_address?page_size=20&page_token=...
Response: { "trades": [...], "next_page_token": "string" | null }
```

**Trade object:**
```json
{
  "id": "uuid",
  "wallet_address": "string",
  "asset": "SOL",
  "side": "buy",
  "quantity": 1.0,
  "order_price": 150.0,
  "filled_price": 150.0,
  "total_value": 150.0,
  "fees": 0.15,
  "created_at": 1700000000000
}
```

---

### Portfolio

```
GET /portfolio/:wallet_address
Response: [
  {
    "wallet_address": "string",
    "asset": "SOL",
    "quantity": 1.0,
    "avg_entry_price": 150.0,
    "current_price": 160.0,
    "unrealized_pnl": 10.0,
    "realized_pnl": 0.0,
    "opened_at": 1700000000000,
    "updated_at": 1700000000000
  }
]
```

---

### Limit Orders

```
POST /orders
Body: {
  "wallet_address": "string",
  "asset": "SOL",
  "side": "buy" | "sell",
  "order_type": "buy_limit" | "stop_loss" | "take_profit",
  "quantity": 1.0,
  "limit_price": 140.0
}

GET /orders/:wallet_address → [LimitOrder]

DELETE /orders/:wallet_address/:id → 204 No Content
```

Limit orders are evaluated every 5 seconds by the background worker:
- `buy_limit` — triggers when `current_price <= limit_price`
- `stop_loss` — triggers when `current_price <= limit_price`
- `take_profit` — triggers when `current_price >= limit_price`

---

### Closed Positions

```
GET /closed-positions/:wallet_address?page_size=20&page_token=...
Response: {
  "positions": [
    {
      "wallet_address": "string",
      "asset": "SOL",
      "quantity": 1.0,
      "avg_entry_price": 150.0,
      "exit_price": 170.0,
      "realized_pnl": 20.0,
      "opened_at": 1700000000000,
      "closed_at": 1700000000000
    }
  ],
  "next_page_token": null
}
```

---

### Portfolio Performance

```
GET /portfolio-performance/:wallet_address?page_size=100&page_token=...
Response: {
  "history": [
    {
      "wallet_address": "string",
      "timestamp": 1700000000000,
      "balance": 10250.0,
      "realized_pnl": 250.0
    }
  ],
  "next_page_token": null
}
```
> A snapshot is recorded automatically after every buy or sell.

---

### Prices

```
GET /prices?tokens=BTC,ETH,SOL
Response: { "BTC": 45000.0, "ETH": 2500.0, "SOL": 150.0 }
```

---

### Leaderboard

```
GET /leaderboard?bucket=global&limit=20
Response: [
  { "bucket": "global", "wallet_address": "string", "total_pnl": 1500.0 }
]
```

---

## Architecture

```
HTTP Request
     │
     ▼
Axum Router (CORS enabled)
     │
     ▼
Route Handler  ──────────────►  MarketDataService (cache)
     │                                   ▲
     ▼                                   │
  Service Layer                    Background Worker
  (TradeService,                   (every 5 seconds)
   PortfolioService, ...)           ├─ Refresh watched asset prices
     │                              └─ Execute triggered limit orders
     ▼
ScyllaDB (batched writes)
```

### Caching

Prices are cached in memory with a 5-second TTL. The background worker proactively refreshes all watched assets (any asset in an open position or recent trade) on each tick. A `fetch_lock` mutex prevents concurrent Binance calls (double-checked locking pattern), so even under high concurrency only one outbound request is made per refresh cycle.

### Atomic Writes

Trade execution uses ScyllaDB logged batches to atomically update multiple tables in one operation:

| Operation | Tables Updated |
|---|---|
| Buy (new position) | `positions` + `trades` + `users` |
| Buy (add to position) | `positions` + `trades` + `users` |
| Sell (partial) | `positions` + `trades` + `users` + `portfolio_performance` |
| Sell (full exit) | `closed_positions` + `positions` (delete) + `trades` + `users` + `portfolio_performance` |

### Database Schema

All timestamps are stored as `BIGINT` (milliseconds since Unix epoch).

| Table | Partition Key | Clustering Key |
|---|---|---|
| `users` | `wallet_address` | — |
| `positions` | `wallet_address` | `asset` |
| `trades` | `wallet_address` | `created_at DESC`, `id` |
| `limit_orders` | `wallet_address` | `id` |
| `closed_positions` | `wallet_address` | `closed_at DESC`, `asset` |
| `portfolio_performance` | `wallet_address` | `timestamp DESC` |
| `leaderboard` | `bucket` | `total_pnl DESC`, `wallet_address` |

## Example Usage

```bash
# Create account
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "alice"}'

# Buy 0.5 BTC with stop-loss and take-profit
curl -X POST http://localhost:8080/trade \
  -H "Content-Type: application/json" \
  -d '{"wallet_address":"alice","asset":"BTC","side":"buy","quantity":0.5,"stop_loss":40000,"take_profit":55000}'

# Check portfolio
curl http://localhost:8080/portfolio/alice

# Sell
curl -X POST http://localhost:8080/trade \
  -H "Content-Type: application/json" \
  -d '{"wallet_address":"alice","asset":"BTC","side":"sell","quantity":0.5}'

# View leaderboard
curl "http://localhost:8080/leaderboard?bucket=global&limit=10"
```
