
CREATE TABLE IF NOT EXISTS paper_trading.portfolio_performance (
    wallet_address TEXT,
    timestamp BIGINT,
    balance DOUBLE,
    realized_pnl DOUBLE,
    PRIMARY KEY (wallet_address, timestamp)
) WITH CLUSTERING ORDER BY (timestamp DESC);