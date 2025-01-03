use crate::i18n::AVAILABLE_LOCALES;
use serde::{Deserialize, Serialize};
use std::fs;

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
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(path)?;
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
        self.guilds
            .allowed
            .iter()
            .find(|g| g.id == guild_id)
            .and_then(|g| g.locale.as_deref())
            .unwrap_or(&self.i18n.default_locale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config() -> (NamedTempFile, Config) {
        let config = Config {
            bot: BotConfig {
                token: "test_token".to_string(),
                default_invite_max_age: 300,
            },
            database: DatabaseConfig {
                path: "test.db".to_string(),
            },
            server: ServerConfig {
                external_url: "http://localhost:8080".to_string(),
                bind: "127.0.0.1:8080".to_string(),
            },
            i18n: I18nConfig {
                default_locale: "en".to_string(),
                available_locales: vec!["en".to_string(), "zh-TW".to_string()],
            },
            guilds: GuildConfig { allowed: vec![] },
        };

        let file = NamedTempFile::new().unwrap();
        write!(
            file.as_file(),
            "{}",
            serde_yaml::to_string(&config).unwrap()
        )
        .unwrap();
        (file, config)
    }

    #[test]
    fn test_config_load() {
        let (file, expected_config) = create_test_config();
        let config = Config::load(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.bot.token, expected_config.bot.token);
        assert_eq!(
            config.i18n.default_locale,
            expected_config.i18n.default_locale
        );
    }

    #[test]
    fn test_invalid_locale() {
        let config = Config {
            i18n: I18nConfig {
                default_locale: "invalid".to_string(),
                available_locales: vec!["invalid".to_string()],
            },
            ..create_test_config().1
        };

        let file = NamedTempFile::new().unwrap();
        write!(
            file.as_file(),
            "{}",
            serde_yaml::to_string(&config).unwrap()
        )
        .unwrap();

        assert!(Config::load(file.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn test_get_guild_locale() {
        let mut config = create_test_config().1;

        // 添加一個測試用的公會
        config.guilds.allowed.push(AllowedGuild {
            id: "123".to_string(),
            name: "Test Guild".to_string(),
            invite_channel: "456".to_string(),
            max_age: None,
            locale: Some("zh-TW".to_string()),
            allowed_roles: vec![],
        });

        // 測試指定公會的語言設定
        assert_eq!(config.get_guild_locale("123"), "zh-TW");

        // 測試未知公會使用預設語言
        assert_eq!(config.get_guild_locale("unknown"), "en");

        // 測試沒有指定語言的公會使用預設語言
        config.guilds.allowed[0].locale = None;
        assert_eq!(config.get_guild_locale("123"), "en");
    }

    #[test]
    fn test_invite_limits() {
        let mut config = create_test_config().1;

        // 添加一個帶有邀請限制的公會
        config.guilds.allowed.push(AllowedGuild {
            id: "123".to_string(),
            name: "Test Guild".to_string(),
            invite_channel: "456".to_string(),
            max_age: Some(7200), // 自定義過期時間
            locale: None,
            allowed_roles: vec![AllowedRole {
                id: "789".to_string(),
                invite_limit: InviteLimit { count: 5, days: 7 },
            }],
        });

        let guild = &config.guilds.allowed[0];

        // 測試自定義過期時間
        assert_eq!(guild.max_age.unwrap(), 7200);

        // 測試角色邀請限制
        let role = &guild.allowed_roles[0];
        assert_eq!(role.invite_limit.count, 5);
        assert_eq!(role.invite_limit.days, 7);
    }

    #[test]
    fn test_default_values() {
        let config = create_test_config().1;

        // 測試預設邀請過期時間
        assert_eq!(config.bot.default_invite_max_age, 300);

        // 測試預設語言設定
        assert_eq!(config.i18n.default_locale, "en");
        assert!(config.i18n.available_locales.contains(&"en".to_string()));
        assert!(config.i18n.available_locales.contains(&"zh-TW".to_string()));
    }
}
