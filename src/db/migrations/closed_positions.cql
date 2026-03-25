USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.closed_positions (
    wallet_address TEXT,
    closed_at BIGINT,
    asset TEXT,
    opened_at BIGINT,
    quantity DOUBLE,
    avg_entry_price DOUBLE,
    exit_price DOUBLE,
    realized_pnl DOUBLE,
    PRIMARY KEY (wallet_address, closed_at, asset)
) WITH CLUSTERING ORDER BY (closed_at DESC);
