USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.limit_orders (
    wallet_address TEXT,
    created_at TIMESTAMP,
    id UUID,
    asset TEXT,
    side TEXT,
    quantity DOUBLE,
    limit_price DOUBLE,
    PRIMARY KEY (wallet_address, created_at, id)
) WITH CLUSTERING ORDER BY (created_at DESC);
