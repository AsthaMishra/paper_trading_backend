use paper_trading_backend::{AppState, config::DatabaseConfig};
fn main() {
    println!("Hello, world!");

    dotenvy::dotenv().ok();
    let db = DatabaseConfig::from_env();

    let app_state = AppState::new(db);
}
