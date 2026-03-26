USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.limit_orders (
    wallet_address TEXT,
    id UUID,
    asset TEXT,
    side TEXT,
    order_type TEXT,
    quantity DOUBLE,
    limit_price DOUBLE,
    created_at TIMESTAMP,
    PRIMARY KEY (wallet_address, id)
);
