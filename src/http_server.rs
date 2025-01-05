use crate::t;
use axum::{
    extract::{Path, State},
    response::Redirect,
    routing::get,
    Router,
};
use poise::serenity_prelude::{self as serenity, builder::CreateInvite};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

use crate::utils::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
}

pub async fn run_server(config: Config, db: SqlitePool) {
    let bind_addr = config.server.bind.clone();
    let app_state = AppState {
        db,
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/invite/:id", get(handle_invite))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr: SocketAddr = bind_addr
        .parse()
        .expect(&t!("en", "errors.server.invalid_address"));
    println!(
        "{}",
        t!(
            "en",
            "server.running",
            HashMap::from([("addr", addr.to_string())])
        )
    );

    axum::serve(
        tokio::net::TcpListener::bind(&addr)
            .await
            .expect(&t!("en", "errors.server.bind_failed")),
        app.into_make_service(),
    )
    .await
    .expect(&t!("en", "errors.server.start_failed"));
}

use axum::http::StatusCode;

pub async fn handle_invite(
    Path(invite_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Redirect, (StatusCode, String)> {
    // Check if invite exists and hasn't been used
    let invite_record = crate::utils::db::get_unused_invite(&state.db, &invite_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                t!(
                    &state.config.i18n.default_locale,
                    "http.errors.internal",
                    HashMap::from([("error", e.to_string())])
                ),
            )
        })?
        .ok_or((
            StatusCode::BAD_REQUEST,
            t!(
                &state.config.i18n.default_locale,
                "http.errors.invalid_invite"
            ),
        ))?;

    // If invite already has a code, redirect to it
    if let Some(code) = invite_record.code {
        return Ok(Redirect::temporary(&format!("https://discord.gg/{}", code)));
    }

    // Get guild configuration
    let guild_config = state
        .config
        .guilds
        .allowed
        .iter()
        .find(|g| g.id == invite_record.guild_id)
        .ok_or((
            StatusCode::BAD_REQUEST,
            t!(
                &state.config.i18n.default_locale,
                "http.errors.server_not_found"
            ),
        ))?;

    let channel_id =
        serenity::ChannelId::new(guild_config.invite_channel.parse().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                t!(
                    &state.config.i18n.default_locale,
                    "http.errors.invalid_channel"
                ),
            )
        })?);

    // Get invite expiration settings
    let max_age = guild_config
        .max_age
        .unwrap_or(state.config.bot.default_invite_max_age);

    // Create Discord invite link
    let http = serenity::Http::new(&state.config.bot.token);
    let invite = channel_id
        .create_invite(
            &http,
            CreateInvite::default()
                .max_age(max_age)
                .max_uses(2)
                .temporary(false),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                t!(
                    &state.config.i18n.default_locale,
                    "http.errors.create_failed",
                    HashMap::from([("error", e.to_string())])
                ),
            )
        })?;

    // Update invite information in database
    crate::utils::db::update_invite_code(&state.db, &invite_id, &invite.code)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                t!(
                    &state.config.i18n.default_locale,
                    "http.errors.update_failed",
                    HashMap::from([("error", e.to_string())])
                ),
            )
        })?;

    Ok(Redirect::temporary(&format!(
        "https://discord.gg/{}",
        invite.code
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::{
        BotConfig, Config, DatabaseConfig, GuildConfig, I18nConfig, ServerConfig,
    };
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn setup_test_app() -> (Router, SqlitePool) {
        let db_url = format!("sqlite:file:{}?mode=memory", Uuid::new_v4());
        let pool = crate::utils::db::create_pool(&db_url).await.unwrap();

        let config = Arc::new(Config {
            bot: BotConfig {
                token: "test_token".to_string(),
                default_invite_max_age: 300,
            },
            database: DatabaseConfig { uri: db_url },
            server: ServerConfig {
                external_url: "http://localhost:8080".to_string(),
                bind: "127.0.0.1:8080".to_string(),
            },
            i18n: I18nConfig {
                default_locale: "en".to_string(),
                available_locales: vec!["en".to_string()],
            },
            guilds: GuildConfig { allowed: vec![] },
        });

        let app_state = AppState {
            db: pool.clone(),
            config,
        };

        let app = Router::new()
            .route("/invite/{id}", get(handle_invite))
            .layer(CorsLayer::permissive())
            .with_state(app_state);

        (app, pool)
    }

    #[tokio::test]
    async fn test_invalid_invite() {
        let (app, _) = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/invite/invalid-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_valid_invite() {
        let (app, pool) = setup_test_app().await;
        let invite_id = Uuid::new_v4().to_string();
        let invite_code = Uuid::new_v4().to_string();

        // Create test invite
        crate::utils::db::create_invite(&pool, &invite_id, "123456789", "987654321")
            .await
            .unwrap();

        crate::utils::db::update_invite_code(&pool, &invite_id, &invite_code)
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/invite/{}", invite_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        dbg!(response.status());
        dbg!(response.headers());
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response
                .headers()
                .get("Location")
                .unwrap()
                .to_str()
                .unwrap(),
            format!("https://discord.gg/{}", invite_code)
        );
    }
}
