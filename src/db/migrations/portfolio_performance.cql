CREATE TABLE IF NOT EXISTS paper_trading.portfolio_performance (
    wallet_address TEXT,
    recorded_at TIMESTAMP,
    balance DOUBLE,
    total_realized_pnl DOUBLE,
    PRIMARY KEY (wallet_address, recorded_at)
) WITH CLUSTERING ORDER BY (recorded_at DESC);