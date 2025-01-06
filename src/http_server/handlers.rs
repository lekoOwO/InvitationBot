use super::server::AppState;
use crate::t;
use crate::utils::config::Config;
use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect},
};
use poise::serenity_prelude::CreateInvite;
use serde::{Deserialize, Serialize};
use serenity::http::Http;
use serenity::model::id::ChannelId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct ConfigResponse {
    config: Config,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct LocalesResponse {
    pub locales: Vec<String>,
}

pub async fn handle_invite(
    Path(invite_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, (StatusCode, String)> {
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

    if let Some(code) = invite_record.code {
        return Ok(Redirect::temporary(&format!("https://discord.gg/{}", code)));
    }

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

    let channel_id = ChannelId::new(guild_config.invite_channel.parse().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            t!(
                &state.config.i18n.default_locale,
                "http.errors.invalid_channel"
            ),
        )
    })?);

    let max_age = guild_config
        .max_age
        .unwrap_or(state.config.bot.default_invite_max_age);

    let http = Http::new(&state.config.bot.token);
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

pub async fn get_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = &state.config;
    (
        StatusCode::OK,
        Json(ConfigResponse {
            config: config.as_ref().clone(),
        }),
    )
}

pub async fn update_config(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ConfigResponse>,
) -> impl IntoResponse {
    println!("Received config: {:?}", payload.config);
    payload
        .config
        .save(std::env::var("CONFIG_PATH").unwrap().as_str())
        .unwrap();

    (
        StatusCode::OK,
        Json(ConfigResponse {
            config: payload.config.clone(),
        }),
    )
}

pub async fn get_locales(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(LocalesResponse {
            locales: crate::i18n::AVAILABLE_LOCALES
                .map(|s| s.to_string())
                .to_vec(),
        }),
    )
}

pub async fn serve_embedded_files(path: Option<Path<String>>) -> impl IntoResponse {
    let path = path.unwrap_or(axum::extract::Path(String::default()));
    let path = if path.is_empty() {
        "index.html".to_string()
    } else {
        path.to_string()
    };

    match crate::public::Assets::get(&path) {
        Some(file) => {
            let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
            (
                StatusCode::OK,
                axum::response::Response::builder()
                    .header("Content-Type", mime_type.as_ref())
                    .body(Body::from(file.data))
                    .unwrap(),
            )
        }
        None => (StatusCode::NOT_FOUND, "Not Found".into_response()),
    }
}
