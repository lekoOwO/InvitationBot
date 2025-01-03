use serde::{Deserialize, Serialize};
use std::fs;
use crate::i18n::AVAILABLE_LOCALES;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub guilds: GuildConfig,
    pub i18n: I18nConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    pub default_locale: String,
    pub available_locales: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub token: String,
    pub default_invite_max_age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub external_url: String,
    pub bind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildConfig {
    pub allowed: Vec<AllowedGuild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedGuild {
    pub id: String,
    pub name: String,
    pub invite_channel: String,
    pub max_age: Option<u32>,
    pub locale: Option<String>,
    pub allowed_roles: Vec<AllowedRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedRole {
    pub id: String,
    pub invite_limit: InviteLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteLimit {
    pub count: i32,
    pub days: i32,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string("data/config.yaml")?;
        let config: Config = serde_yaml::from_str(&content)?;

        // Validate locales
        for locale in &config.i18n.available_locales {
            if !AVAILABLE_LOCALES.contains(&locale.as_str()) {
                return Err(format!("Unsupported locale: {}", locale).into());
            }
        }

        Ok(config)
    }

    pub fn get_guild_locale(&self, guild_id: &str) -> &str {
        self.guilds.allowed.iter()
            .find(|g| g.id == guild_id)
            .and_then(|g| g.locale.as_deref())
            .unwrap_or(&self.i18n.default_locale)
    }
} 