pub const DEFAULT_DATABASE_URL: &str = "sqlite://fumo.db";

/// Gets the Discord bot token from environment
pub fn discord_token() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::var("DISCORD_TOKEN")
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

/// Gets the database URL from environment
pub fn database_url() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    match dotenvy::var("DATABASE_URL") {
        Ok(value) => Ok(Some(value)),
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(None),
        Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
    }
}
