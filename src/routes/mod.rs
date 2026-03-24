pub mod trade;
use axum::{Router, routing::Route};
pub use trade::*;

pub mod portfolio;
pub use portfolio::*;

pub mod leaderboard;
pub use leaderboard::*;

pub mod user;
pub use user::*;

pub mod portfolio_performance;
pub use portfolio_performance::*;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(user::routes())
        .merge(trade::routes())
        .merge(portfolio::routes())
        .merge(leaderboard::routes())
}
