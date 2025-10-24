use config::ConfigError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub links_path: String,
    pub default_topic: String,
    pub draft_location: String,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| ConfigError::Message("Could not find home directory".to_string()))?;

    let config_path = home_dir.join(".config").join("rott").join("config.yaml");
    let config_builder = config::Config::builder()
        .add_source(config::File::with_name(config_path.to_str().unwrap()).required(false))
        .add_source(config::Environment::with_prefix("APP"));

    let config = config_builder.build()?;

    config.try_deserialize::<AppConfig>()
}
