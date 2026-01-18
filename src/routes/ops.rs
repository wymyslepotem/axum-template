use axum::{Router, routing::get};

use crate::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/health", get(handlers::ops::get_health))
}
