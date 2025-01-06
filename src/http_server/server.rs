use super::handlers::{get_config, get_locales, handle_invite, serve_embedded_files};
use crate::http_server::handlers::update_config;
use crate::utils::config::Config;
use axum::{routing::get, Router};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
}

pub async fn run_server(config: Config, db: SqlitePool) {
    let bind_addr = config.server.bind.clone();
    let app_state = Arc::new(AppState {
        db,
        config: Arc::new(config),
    });

    let app = Router::new()
        .route("/invite/{id}", get(handle_invite))
        .route("/config", get(get_config).post(update_config))
        .route("/locales", get(get_locales))
        .route("/", get(serve_embedded_files))
        .route("/{*path}", get(serve_embedded_files))
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    let addr: SocketAddr = bind_addr
        .parse()
        .expect(&crate::t!("en", "errors.server.invalid_address"));
    println!(
        "{}",
        crate::t!(
            "en",
            "server.running",
            std::collections::HashMap::from([("addr", addr.to_string())])
        )
    );

    axum::serve(
        tokio::net::TcpListener::bind(&addr)
            .await
            .expect(&crate::t!("en", "errors.server.bind_failed")),
        app.into_make_service(),
    )
    .await
    .expect(&crate::t!("en", "errors.server.start_failed"));
}
