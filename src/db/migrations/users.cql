CREATE KEYSPACE IF NOT EXISTS paper_trading WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

USE paper_trading;

CREATE TABLE IF NOT EXISTS paper_trading.users (
                wallet_address TEXT PRIMARY KEY,
                starting_balance DOUBLE,
                current_balance DOUBLE,
                total_realized_pnl DOUBLE,
                created_at BIGINT,
                total_trades INT,
                winning_trades INT,
                best_trade DOUBLE,
                worst_trade DOUBLE
            );