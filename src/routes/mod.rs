use axum::Router;

pub const DEFAULT_PAGE_SIZE: i32 = 20;

pub mod trade;
pub use trade::*;

pub mod portfolio;
pub use portfolio::*;

pub mod leaderboard;
pub use leaderboard::*;

pub mod user;
pub use user::*;

pub mod portfolio_performance;
pub use portfolio_performance::*;

pub mod closed_positions;
pub use closed_positions::*;

pub mod orders;
pub use orders::*;

pub mod prices;
pub use prices::*;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(user::routes())
        .merge(trade::routes())
        .merge(portfolio::routes())
        .merge(leaderboard::routes())
        .merge(portfolio_performance::routes())
        .merge(closed_positions::routes())
        .merge(orders::routes())
        .merge(prices::routes())
}
