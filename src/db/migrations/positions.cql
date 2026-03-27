USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.positions (
    wallet_address TEXT,
    asset TEXT,
    quantity DOUBLE,
    avg_entry_price DOUBLE,
    realized_pnl DOUBLE,
    opened_at BIGINT,
    updated_at BIGINT,
    PRIMARY KEY (wallet_address, asset)
);
