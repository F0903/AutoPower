#[derive(Debug)]
pub enum ConfigError {
    CouldNotLoadOrCreate,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ConfigError::CouldNotLoadOrCreate => "Could not load or create config!",
        })
    }
}

impl std::error::Error for ConfigError {}
