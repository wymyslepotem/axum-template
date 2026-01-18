use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvDto {
    Development,
    Production,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub ok: bool,
    pub env: EnvDto,
}
