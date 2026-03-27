USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.trades (
    wallet_address TEXT,
    created_at BIGINT,
    id UUID,
    asset TEXT,
    side TEXT,
    quantity DOUBLE,
    order_price DOUBLE,
    filled_price DOUBLE,
    total_value DOUBLE,
    fees DOUBLE,
    PRIMARY KEY (wallet_address, created_at, id)
) WITH CLUSTERING ORDER BY (created_at DESC);