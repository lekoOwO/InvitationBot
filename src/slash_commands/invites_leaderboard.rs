use crate::{Context, Error, t};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use poise::CreateReply;
use std::collections::HashMap;

/// View the invite leaderboard
#[poise::command(slash_command, guild_only)]
pub async fn invites_leaderboard(
    ctx: Context<'_>,
    #[description = "Days to look back (default: 30)"] days: Option<i32>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild = ctx.guild().unwrap().clone();
    let locale = ctx.data().config.get_guild_locale(&guild_id.to_string());
    let days = days.unwrap_or(30);

    let entries = crate::utils::db::get_invite_leaderboard(
        &ctx.data().db,
        &guild_id.to_string(),
        days
    ).await?;

    if entries.is_empty() {
        let mut params = HashMap::new();
        params.insert("days", days.to_string());

        let embed = CreateEmbed::default()
            .title(t!(locale, "commands.invites_leaderboard.errors.no_invites.title"))
            .description(t!(locale, "commands.invites_leaderboard.errors.no_invites.description", params))
            .color(0xFF3333)
            .footer(CreateEmbedFooter::new(t!(locale, "commands.invites_leaderboard.errors.no_invites.footer")));
        
        let reply = CreateReply::default()
            .embed(embed)
            .ephemeral(true);
        ctx.send(reply).await?;
        return Ok(());
    }

    let mut description = String::new();
    for (index, entry) in entries.iter().enumerate() {
        description.push_str(&format!(
            "**#{} →** <@{}> [{} invites]\n\n",
            index + 1,
            entry.creator_id,
            entry.invite_count
        ));
    }

    let mut params = HashMap::new();
    params.insert("guild", guild.name.clone());

    let embed = CreateEmbed::default()
        .title(t!(locale, "commands.invites_leaderboard.success.title", params))
        .description(description)
        .color(0x4CACEE)
        .thumbnail(guild.icon_url().unwrap_or_default())
        .footer(CreateEmbedFooter::new(t!(locale, "commands.invites_leaderboard.success.footer")));

    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
} 