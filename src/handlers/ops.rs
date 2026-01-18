use axum::{Json, extract::State};

use crate::dto::ops::{EnvDto, HealthResponse};
use crate::error::AppError;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Service is healthy", body = HealthResponse)),
    tag = "ops"
)]
pub async fn get_health(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    let env = match state.settings.app_env {
        crate::config::AppEnv::Development => EnvDto::Development,
        crate::config::AppEnv::Production => EnvDto::Production,
    };

    Ok(Json(HealthResponse { ok: true, env }))
}
