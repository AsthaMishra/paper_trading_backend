USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.leaderboard (
                bucket TEXT,
                total_pnl DOUBLE,
                wallet_address TEXT,
                PRIMARY KEY (bucket, total_pnl, wallet_address)
            ) WITH CLUSTERING ORDER BY (total_pnl DESC);