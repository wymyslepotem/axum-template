use axum::Router;
use utoipa::OpenApi;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(crate::handlers::ops::get_health),
    components(schemas(crate::dto::ops::HealthResponse)),
    tags((name = "ops", description = "Operational endpoints"))
)]
pub struct ApiDoc;

pub fn router() -> Router<AppState> {
    let swagger: Router = utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
        .url("/api-doc/openapi.json", ApiDoc::openapi())
        .into();

    // Convert Router<()> -> Router<AppState> so it can be merged.
    swagger.with_state(())
}
