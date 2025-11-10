pub const DEFAULT_DATABASE_URL: &str = "sqlite://fumo.db";

type EnvError = Box<dyn std::error::Error + Send + Sync>;
type EnvResult<T> = Result<T, EnvError>;

/// Gets the Discord bot token from environment
pub fn discord_token() -> EnvResult<String> {
    dotenvy::var("DISCORD_TOKEN").map_err(|e| Box::new(e) as EnvError)
}

/// Gets the database URL from environment
pub fn database_url() -> EnvResult<Option<String>> {
    match dotenvy::var("DATABASE_URL") {
        Ok(value) => Ok(Some(value)),
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(None),
        Err(e) => Err(Box::new(e) as EnvError),
    }
}

/// Comma-separated list of Discord user IDs allowed to usar comandos owners_only
pub fn owner_ids() -> EnvResult<Vec<u64>> {
    match dotenvy::var("FUMO_OWNERS_IDS") {
        Ok(value) => {
            let mut ids = Vec::new();
            for raw in value.split(',') {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let parsed: u64 = trimmed.parse().map_err(|err| Box::new(err) as EnvError)?;
                ids.push(parsed);
            }
            Ok(ids)
        }
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(Vec::new()),
        Err(e) => Err(Box::new(e) as EnvError),
    }
}
