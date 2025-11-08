use std::env;
use std::fmt;
use std::sync::{Arc, LazyLock};

use dotenvy::Error as DotenvyError;

pub const DEFAULT_DATABASE_URL: &str = "sqlite://fumo.db";

static DISCORD_TOKEN: LazyLock<Result<String, EnvVarError>> =
    LazyLock::new(|| required_non_empty("DISCORD_TOKEN"));
static DATABASE_URL: LazyLock<Result<Option<String>, EnvVarError>> =
    LazyLock::new(|| optional_non_empty("DATABASE_URL"));

#[derive(Clone, Debug)]
pub enum EnvVarError {
    Missing {
        key: &'static str,
    },
    Empty {
        key: &'static str,
    },
    NotUnicode {
        key: &'static str,
    },
    Dotenv {
        key: &'static str,
        source: Arc<DotenvyError>,
    },
}

impl fmt::Display for EnvVarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvVarError::Missing { key } => write!(f, "environment variable {key} is missing"),
            EnvVarError::Empty { key } => write!(f, "environment variable {key} is empty"),
            EnvVarError::NotUnicode { key } => {
                write!(f, "environment variable {key} contains invalid unicode")
            }
            EnvVarError::Dotenv { key, source } => {
                write!(f, "unable to read environment variable {key}: {source}")
            }
        }
    }
}

impl std::error::Error for EnvVarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EnvVarError::Dotenv { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

pub fn discord_token() -> Result<String, EnvVarError> {
    match &*DISCORD_TOKEN {
        Ok(value) => Ok(value.clone()),
        Err(err) => Err(err.clone()),
    }
}

pub fn database_url() -> Result<Option<String>, EnvVarError> {
    match &*DATABASE_URL {
        Ok(value) => Ok(value.clone()),
        Err(err) => Err(err.clone()),
    }
}

fn required_non_empty(key: &'static str) -> Result<String, EnvVarError> {
    optional_non_empty(key)?.ok_or(EnvVarError::Missing { key })
}

fn optional_non_empty(key: &'static str) -> Result<Option<String>, EnvVarError> {
    match dotenvy::var(key) {
        Ok(value) => {
            if value.trim().is_empty() {
                Err(EnvVarError::Empty { key })
            } else {
                Ok(Some(value))
            }
        }
        Err(DotenvyError::EnvVar(env::VarError::NotPresent)) => Ok(None),
        Err(DotenvyError::EnvVar(env::VarError::NotUnicode(_))) => {
            Err(EnvVarError::NotUnicode { key })
        }
        Err(err) => Err(EnvVarError::Dotenv {
            key,
            source: Arc::new(err),
        }),
    }
}
