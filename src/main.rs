use paper_trading_backend::{AppState, config::DatabaseConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("Hello, world!");

    dotenvy::dotenv().ok();

    let db_confifg = DatabaseConfig::from_env();
    let db = db_confifg.create_session().await?;

    let app_state = AppState::new(db);

    Ok(())
}
