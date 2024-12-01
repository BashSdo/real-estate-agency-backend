//! [`Config`]-related definitions.

use std::time;

use config::{builder::DefaultState, ConfigBuilder, ConfigError};
use serde::Deserialize;
use smart_default::SmartDefault;

/// Application configuration.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// Server configuration.
    pub server: Server,

    /// Service configuration.
    pub service: Service,

    /// Postgres configuration.
    pub postgres: Postgres,

    /// Log configuration.
    pub log: Log,
}

impl Config {
    /// Creates a new [`Config`] by:
    /// - loading it from the provided `path` (if any);
    /// - merging it with the environment variables (if any);
    /// - using default values for missing fields.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn new(path: impl AsRef<str>) -> Result<Self, ConfigError> {
        ConfigBuilder::<DefaultState>::default()
            .add_source(config::File::with_name(path.as_ref()).required(false))
            .add_source(config::Environment::with_prefix("CONF").separator("."))
            .build()?
            .try_deserialize()
    }
}

/// Server configuration.
#[derive(Clone, Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Server {
    /// Host to bind the server to.
    #[default("0.0.0.0".to_owned())]
    pub host: String,

    /// Port to bind the server to.
    #[default(8080)]
    pub port: u16,

    /// [CORS] configuration.
    ///
    /// [CORS]: https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS
    pub cors: Cors,
}

/// [CORS] configuration.
///
/// [CORS]: https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS
#[derive(Clone, Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Cors {
    /// List of allowed origins.
    #[default(vec!["*".to_owned()])]
    pub origins: Vec<String>,
}

/// Service configuration.
#[derive(Clone, Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Service {
    /// [JWT] secret.
    ///
    /// [JWT]: https://wikipedia.org/wiki/JSON_Web_Token
    #[default("secret".to_owned())]
    pub jwt_secret: String,

    /// Service tasks configuration.
    pub tasks: Tasks,
}

impl From<Service> for service::Config {
    fn from(value: Service) -> Self {
        let Service {
            jwt_secret,
            tasks: Tasks {
                clean_unused_realties,
            },
        } = value;
        Self {
            jwt_encoding_key: jsonwebtoken::EncodingKey::from_secret(
                jwt_secret.as_bytes(),
            ),
            jwt_decoding_key: jsonwebtoken::DecodingKey::from_secret(
                jwt_secret.as_bytes(),
            ),
            clean_unused_realties:
                service::task::clean_unused_realties::Config {
                    interval: clean_unused_realties.interval,
                    timeout: clean_unused_realties.timeout,
                },
        }
    }
}

/// Service tasks configuration.
#[derive(Clone, Copy, Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Tasks {
    /// `CleanUnusedRealties` task configuration.
    pub clean_unused_realties: Task,
}

/// Service task configuration.
#[derive(Clone, Copy, Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Task {
    /// Task execution interval.
    #[default(time::Duration::from_secs(60 * 60))]
    #[serde(with = "humantime_serde")]
    pub interval: time::Duration,

    /// Timeout after which the entities will be considered stale.
    #[default(time::Duration::from_secs(60 * 60 * 24))]
    #[serde(with = "humantime_serde")]
    pub timeout: time::Duration,
}

/// Postgres configuration.
#[derive(Clone, Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Postgres {
    /// Host to connect to.
    #[default("127.0.0.1".to_owned())]
    pub host: String,

    /// Port to connect to.
    #[default(5432)]
    pub port: u16,

    /// User to connect as.
    #[default("postgres".to_owned())]
    pub user: String,

    /// Password to connect with.
    #[default("postgres".to_owned())]
    pub password: String,

    /// Database name to connect to.
    #[default("postgres".to_owned())]
    pub dbname: String,
}

impl From<Postgres> for service::infra::postgres::Config {
    fn from(value: Postgres) -> Self {
        let Postgres {
            host,
            port,
            user,
            password,
            dbname,
        } = value;

        Self {
            host: Some(host),
            port: Some(port),
            user: Some(user),
            password: Some(password),
            dbname: Some(dbname),
            ..Self::default()
        }
    }
}

/// Log configuration.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Log {
    /// Log level.
    pub level: LogLevel,
}

/// Log level.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LogLevel {
    /// Designates very low priority, often extremely verbose, information.
    Trace,

    /// Designates lower priority information.
    Debug,

    /// Designates useful information.
    #[default]
    Info,

    /// Designates hazardous situations.
    Warn,

    /// Designates very serious errors.
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => Self::TRACE,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}
