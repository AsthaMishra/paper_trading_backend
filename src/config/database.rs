use std::sync::Arc;

use scylla::client::{session::Session, session_builder::SessionBuilder};
use serde::{Deserialize, Serialize};

const MIGRATIONS: &[(&str, &str)] = &[
    ("users", include_str!("../db/migrations/users.cql")),
    ("trades", include_str!("../db/migrations/trades.cql")),
    ("positions", include_str!("../db/migrations/positions.cql")),
    (
        "leaderboard",
        include_str!("../db/migrations/leaderboard.cql"),
    ),
    (
        "portfolio_performance",
        include_str!("../db/migrations/portfolio_performance.cql"),
    ),
    (
        "closed_positions",
        include_str!("../db/migrations/closed_positions.cql"),
    ),
    (
        "limit_orders",
        include_str!("../db/migrations/limit_orders.cql"),
    ),
];

#[derive(Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub hosts: Vec<String>,
    pub keyspace: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub port: u16,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            hosts: std::env::var("DB_HOSTS")
                .unwrap_or_else(|_| "127.0.0.1".to_string())
                .split(",")
                .map(|h| h.to_string())
                .collect(),

            keyspace: std::env::var("DB_KEYSPACE").unwrap(),

            username: std::env::var("DB_USER").ok(),

            password: std::env::var("DB_PASS").ok(),

            port: std::env::var("DB_PORT")
                .unwrap_or_else(|_| "9042".to_string())
                .parse()
                .unwrap_or(9042),
        }
    }

    pub async fn create_session(&self) -> Result<Arc<Session>, Box<dyn std::error::Error>> {
        let mut session_builder = SessionBuilder::new();

        let hosts: Vec<String> = self.hosts.iter().map(|h| h.to_string()).collect();
        for host in hosts {
            let node_addr = format!("{}:{}", host, self.port);
            session_builder = session_builder.known_node(node_addr);
        }

        let session: Session = session_builder.build().await?;
        self.create_keyspace(&session).await?;

        session.use_keyspace(&self.keyspace, false).await?;
        self.run_migrations(&session).await?;
        Ok(Arc::new(session))
    }

    async fn create_keyspace(&self, session: &Session) -> Result<(), Box<dyn std::error::Error>> {
        let query = format!(
            "CREATE KEYSPACE IF NOT EXISTS {}
            WITH replication = {{
                'class': 'SimpleStrategy',
                'replication_factor': '1'
            }}",
            self.keyspace
        );

        session.query_unpaged(query, &[]).await?;
        println!("✅ Keyspace '{}' ready", self.keyspace);
        Ok(())
    }

    async fn run_migrations(&self, session: &Session) -> Result<(), Box<dyn std::error::Error>> {
        for (name, content) in MIGRATIONS {
            for statement in content.split(';') {
                let statement = statement.trim();
                if statement.is_empty() || statement.to_uppercase().starts_with("USE") {
                    continue;
                }
                session
                    .query_unpaged(statement, &[])
                    .await
                    .map_err(|e| format!("migration '{}' failed: {}", name, e))?;
            }
            println!("✅ Migration '{}' applied", name);
        }
        Ok(())
    }
}
