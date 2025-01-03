use crate::{t, Context, Error};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use poise::CreateReply;
use std::collections::HashMap;
use uuid::Uuid;

/// Create an invite link
#[poise::command(slash_command, guild_only)]
pub async fn invites(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let guild = ctx.guild().unwrap().clone();
    let member = ctx.author_member().await.unwrap();
    let locale = ctx.data().config.get_guild_locale(&guild_id.to_string());

    // Check if guild is allowed
    let guild_config = match ctx
        .data()
        .config
        .guilds
        .allowed
        .iter()
        .find(|g| g.id == guild_id.to_string())
    {
        Some(config) => config,
        None => {
            let embed = CreateEmbed::default()
                .title(t!(
                    locale,
                    "commands.invites.errors.server_not_allowed.title"
                ))
                .description(t!(
                    locale,
                    "commands.invites.errors.server_not_allowed.description"
                ))
                .color(0xFF3333)
                .footer(CreateEmbedFooter::new(t!(
                    locale,
                    "commands.invites.errors.server_not_allowed.footer"
                )));

            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    // Check if user has allowed role
    let role_with_limit = match member.roles.iter().find_map(|role_id| {
        guild_config
            .allowed_roles
            .iter()
            .find(|allowed_role| allowed_role.id == role_id.to_string())
    }) {
        Some(role) => role,
        None => {
            let embed = CreateEmbed::default()
                .title(t!(
                    locale,
                    "commands.invites.errors.missing_permissions.title"
                ))
                .description(t!(
                    locale,
                    "commands.invites.errors.missing_permissions.description"
                ))
                .color(0xFF3333)
                .footer(CreateEmbedFooter::new(t!(
                    locale,
                    "commands.invites.errors.missing_permissions.footer"
                )));

            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    // Check invite limit
    let used_invites = crate::utils::db::count_used_invites(
        &ctx.data().db,
        &ctx.author().id.to_string(),
        &guild_id.to_string(),
        role_with_limit.invite_limit.days,
    )
    .await?;

    if used_invites >= role_with_limit.invite_limit.count as i64 {
        let mut params = HashMap::new();
        params.insert("count", role_with_limit.invite_limit.count.to_string());
        params.insert("days", role_with_limit.invite_limit.days.to_string());
        params.insert("used", used_invites.to_string());
        params.insert(
            "remaining",
            (role_with_limit.invite_limit.count as i64 - used_invites).to_string(),
        );

        let embed = CreateEmbed::default()
            .title(t!(locale, "commands.invites.errors.limit_reached.title"))
            .description(format!(
                "{}\n\n**{}**:\n• {}\n• {}",
                t!(
                    locale,
                    "commands.invites.errors.limit_reached.description",
                    params.clone()
                ),
                t!(locale, "commands.invites.errors.limit_reached.status"),
                t!(
                    locale,
                    "commands.invites.errors.limit_reached.used",
                    params.clone()
                ),
                t!(
                    locale,
                    "commands.invites.errors.limit_reached.remaining",
                    params
                )
            ))
            .color(0xFF3333)
            .footer(CreateEmbedFooter::new(t!(
                locale,
                "commands.invites.errors.limit_reached.footer"
            )));

        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let invite_id = Uuid::new_v4().to_string();

    // Record basic information
    crate::utils::db::create_invite(
        &ctx.data().db,
        &invite_id,
        &guild_id.to_string(),
        &ctx.author().id.to_string(),
    )
    .await?;

    let bot_invite_url = format!(
        "{}/invite/{}",
        ctx.data().config.server.external_url,
        invite_id
    );

    let mut params = HashMap::new();
    params.insert("guild", guild.name.clone());
    params.insert("count", role_with_limit.invite_limit.count.to_string());
    params.insert("days", role_with_limit.invite_limit.days.to_string());
    params.insert("used", used_invites.to_string());
    params.insert(
        "remaining",
        (role_with_limit.invite_limit.count as i64 - used_invites).to_string(),
    );

    let embed = CreateEmbed::default()
        .title(t!(locale, "commands.invites.success.title"))
        .description(format!(
            "{}\n\n{}\n\n**{}**:\n• {}\n• {}",
            t!(
                locale,
                "commands.invites.success.description",
                params.clone()
            ),
            bot_invite_url,
            t!(locale, "commands.invites.success.limits"),
            t!(
                locale,
                "commands.invites.success.invites_per_days",
                params.clone()
            ),
            t!(locale, "commands.invites.success.used_remaining", params)
        ))
        .color(0x4CACEE)
        .thumbnail(guild.icon_url().unwrap_or_default())
        .footer(CreateEmbedFooter::new(t!(
            locale,
            "commands.invites.success.footer"
        )));

    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
}
