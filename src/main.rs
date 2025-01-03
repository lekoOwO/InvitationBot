use poise::serenity_prelude::{self as serenity, async_trait};
use poise::serenity_prelude::CreateEmbedFooter;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

mod slash_commands;
mod utils;
mod http_server;
mod handlers;

use utils::config::Config;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
pub struct Data {
    db: SqlitePool,
    config: utils::config::Config,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            let locale = "en";
            panic!("{}", t!(locale, "errors.setup", HashMap::from([
                ("error", format!("{:?}", error))
            ])));
        },
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Command error: {:?}", error);
            let guild_id = ctx.guild_id().unwrap();
            let locale = ctx.data().config.get_guild_locale(&guild_id.to_string());

            let embed = poise::serenity_prelude::CreateEmbed::default()
                .title(t!(locale, "errors.command.title"))
                .description(t!(locale, "errors.command.description"))
                .color(0xFF3333)
                .footer(CreateEmbedFooter::new(t!(locale, "errors.command.footer")));
            
            let reply = poise::CreateReply::default()
                .embed(embed)
                .ephemeral(true);
            let _ = ctx.send(reply).await;
        }
        error => {
            let locale = "en";
            println!("{}", t!(locale, "errors.unknown", HashMap::from([
                ("error", format!("{:?}", error))
            ])));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::load()?;
    
    // Initialize database connection pool
    let db = utils::db::create_pool(&config.database.path).await?;

    // Start HTTP server
    let server_config = config.clone();
    let server_db = db.clone();
    tokio::spawn(async move {
        http_server::run_server(server_config, server_db).await;
    });

    let db_clone = db.clone();
    let config_clone1 = config.clone();
    let config_clone2 = config.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                slash_commands::ping::ping(),
                slash_commands::invites::invites(),
                slash_commands::inviter::inviter(),
                slash_commands::invites_leaderboard::invites_leaderboard(),
            ],
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let locale = config_clone1.i18n.default_locale.as_str();
                println!("{}", t!(locale, "bot.logged_in", HashMap::from([
                    ("name", _ready.user.name.clone())
                ])));
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db, config: config_clone1 })
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged() 
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_INVITES;

    let mut client = serenity::ClientBuilder::new(&config.bot.token, intents)
        .framework(framework)
        .event_handler(Handler { data: Data { db: db_clone, config: config_clone2 } })
        .await?;

    client.start().await?;

    Ok(())
}

struct Handler {
    data: Data,
}

#[async_trait]
impl serenity::EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: serenity::Context, new_member: serenity::Member) {
        handlers::handle_guild_member_addition(&ctx, new_member.guild_id, &new_member, &self.data).await;
    }
}

#[macro_export]
macro_rules! t {
    ($locale:expr, $key:expr) => {
        crate::utils::i18n::get_text($locale, $key, None)
    };
    ($locale:expr, $key:expr, $params:expr) => {
        crate::utils::i18n::get_text($locale, $key, Some($params))
    };
}
