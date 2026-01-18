use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone)]
pub struct Settings {
    pub http_host: IpAddr,
    pub http_port: u16,

    pub app_env: AppEnv,
    pub log_format: LogFormat,
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

        Ok(Self {
            http_host,
            http_port,
            app_env,
            log_format: if app_env == AppEnv::Production {
                LogFormat::Json
            } else {
                log_format
            },
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.http_host, self.http_port)
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
