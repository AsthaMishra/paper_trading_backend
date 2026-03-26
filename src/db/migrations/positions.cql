USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.positions (
    wallet_address TEXT,
    asset TEXT,
    quantity DOUBLE,
    avg_entry_price DOUBLE,
    realized_pnl DOUBLE,
    opened_at TIMESTAMP,
    updated_at TIMESTAMP,
    PRIMARY KEY (wallet_address, asset)
);
