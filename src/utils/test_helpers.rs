use sqlx::SqlitePool;
use uuid::Uuid;

pub struct TestContext {
    pub db: SqlitePool,
    pub config: crate::utils::config::Config,
}

impl TestContext {
    pub async fn new() -> Self {
        let db_url = format!("sqlite:file:{}?mode=memory", Uuid::new_v4());
        let pool = crate::utils::db::create_pool(&db_url).await.unwrap();

        let config = crate::utils::config::Config {
            bot: crate::utils::config::BotConfig {
                token: "test_token".to_string(),
                default_invite_max_age: 300,
                default_min_member_age: 5184000,
            },
            database: crate::utils::config::DatabaseConfig { uri: db_url },
            server: crate::utils::config::ServerConfig {
                external_url: "http://localhost:8080".to_string(),
                bind: "127.0.0.1:8080".to_string(),
            },
            i18n: crate::utils::config::I18nConfig {
                default_locale: "en".to_string(),
                available_locales: vec!["en".to_string()],
            },
            guilds: crate::utils::config::GuildConfig { allowed: vec![] },
        };

        Self { db: pool, config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let test_ctx = TestContext::new().await;
        assert!(test_ctx.db.acquire().await.is_ok());
        assert_eq!(test_ctx.config.i18n.default_locale, "en");
    }
}
