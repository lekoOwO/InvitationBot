use axum::{
    extract::{Path, State},
    response::Redirect,
    routing::get,
    Router,
};
use sqlx::SqlitePool;
use std::{sync::Arc, net::SocketAddr};
use tower_http::cors::CorsLayer;
use poise::serenity_prelude::{
    self as serenity,
    builder::CreateInvite,
};
use crate::t;
use std::collections::HashMap;

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

    let addr: SocketAddr = bind_addr.parse()
        .expect(&t!("en", "errors.server.invalid_address"));
    println!("{}", t!("en", "server.running", HashMap::from([("addr", addr.to_string())])));

    axum::serve(
        tokio::net::TcpListener::bind(&addr).await
            .expect(&t!("en", "errors.server.bind_failed")),
        app.into_make_service(),
    )
    .await
    .expect(&t!("en", "errors.server.start_failed"));
}

pub async fn handle_invite(
    Path(invite_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Redirect, String> {
    // Check if invite exists and hasn't been used
    let invite_record = crate::utils::db::get_unused_invite(&state.db, &invite_id)
        .await
        .map_err(|e| t!(&state.config.i18n.default_locale, "http.errors.internal", HashMap::from([("error", e.to_string())])))?
        .ok_or(t!(&state.config.i18n.default_locale, "http.errors.invalid_invite"))?;

    // Get guild configuration
    let guild_config = state.config.guilds.allowed.iter()
        .find(|g| g.id == invite_record.guild_id)
        .ok_or(t!(&state.config.i18n.default_locale, "http.errors.server_not_found"))?;

    let channel_id = serenity::ChannelId::new(
        guild_config.invite_channel.parse()
            .map_err(|_| t!(&state.config.i18n.default_locale, "http.errors.invalid_channel"))?
    );

    // Get invite expiration settings
    let max_age = guild_config.max_age
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
        .map_err(|e| t!(&state.config.i18n.default_locale, "http.errors.create_failed", HashMap::from([("error", e.to_string())])))?;

    // Update invite information in database
    crate::utils::db::update_invite_code(&state.db, &invite_id, &invite.code)
        .await
        .map_err(|e| t!(&state.config.i18n.default_locale, "http.errors.update_failed", HashMap::from([("error", e.to_string())])))?;

    Ok(Redirect::temporary(&format!(
        "https://discord.gg/{}",
        invite.code
    )))
} 