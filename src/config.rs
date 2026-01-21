use axum::http::HeaderValue;
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;

#[derive(Debug, Clone)]
pub struct Settings {
    pub http_host: IpAddr,
    pub http_port: u16,

    pub app_env: AppEnv,
    pub log_format: LogFormat,

    pub cors: CorsConfig,
    pub ratelimit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

#[derive(Debug, Clone)]
pub enum CorsConfig {
    Disabled,
    Any,
    AllowList(Vec<HeaderValue>),
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub rps: NonZeroU32,
    pub burst: NonZeroU32,
    pub trust_proxy: bool,
}

impl Settings {
    pub fn from_env() -> Result<Self, String> {
        let http_host = std::env::var("HTTP_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .parse::<IpAddr>()
            .map_err(|e| format!("HTTP_HOST is invalid: {e}"))?;

        let http_port = std::env::var("HTTP_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| format!("HTTP_PORT is invalid: {e}"))?;

        let app_env = match std::env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "prod" | "production" => AppEnv::Production,
            _ => AppEnv::Development,
        };

        let log_format = match std::env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            _ => LogFormat::Pretty,
        };

        let cors = parse_cors_origins(std::env::var("APP_CORS_ORIGINS").unwrap_or_default())?;

        let ratelimit = match std::env::var("APP_RATELIMIT_RPS") {
            Ok(value) => {
                let rps = parse_nonzero_u32("APP_RATELIMIT_RPS", &value)?;
                let burst = match std::env::var("APP_RATELIMIT_BURST") {
                    Ok(burst_value) => parse_nonzero_u32("APP_RATELIMIT_BURST", &burst_value)?,
                    Err(_) => NonZeroU32::new(20).expect("default burst must be non-zero"),
                };
                let trust_proxy = match std::env::var("APP_RATELIMIT_TRUST_PROXY") {
                    Ok(value) => parse_bool("APP_RATELIMIT_TRUST_PROXY", &value)?,
                    Err(_) => false,
                };

                Some(RateLimitConfig {
                    rps,
                    burst,
                    trust_proxy,
                })
            }
            Err(_) => None,
        };

        Ok(Self {
            http_host,
            http_port,
            app_env,
            log_format: if app_env == AppEnv::Production {
                LogFormat::Json
            } else {
                log_format
            },
            cors,
            ratelimit,
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.http_host, self.http_port)
    }
}

fn parse_cors_origins(value: String) -> Result<CorsConfig, String> {
    let raw = value.trim();
    if raw.is_empty() {
        return Ok(CorsConfig::Disabled);
    }

    if raw == "*" {
        return Ok(CorsConfig::Any);
    }

    let mut origins = Vec::new();
    for origin in raw
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
    {
        if origin == "*" {
            return Err("APP_CORS_ORIGINS cannot include '*' when using an allowlist".to_string());
        }
        let header_value = HeaderValue::from_str(origin)
            .map_err(|e| format!("APP_CORS_ORIGINS entry is invalid: {e}"))?;
        origins.push(header_value);
    }

    if origins.is_empty() {
        return Ok(CorsConfig::Disabled);
    }

    Ok(CorsConfig::AllowList(origins))
}

fn parse_nonzero_u32(name: &str, value: &str) -> Result<NonZeroU32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|e| format!("{name} is invalid: {e}"))?;
    NonZeroU32::new(parsed).ok_or_else(|| format!("{name} must be greater than zero"))
}

fn parse_bool(name: &str, value: &str) -> Result<bool, String> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(format!("{name} must be a boolean value")),
    }
}

pub fn init_tracing(settings: &Settings) {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info"));

    if settings.log_format == LogFormat::Json {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .init();
    }
}
