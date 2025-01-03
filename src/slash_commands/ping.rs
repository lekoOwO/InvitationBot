use crate::{Context, Error, t};
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;

/// Check if the bot is alive
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let locale = ctx.data().config.get_guild_locale(&guild_id.to_string());

    let embed = CreateEmbed::default()
        .title(t!(locale, "commands.ping.response.title"))
        .description(t!(locale, "commands.ping.response.description"))
        .color(0x4CACEE);
    
    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;
    Ok(())
} 