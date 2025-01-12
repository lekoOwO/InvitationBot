use crate::utils::config::AllowedRole;
use crate::{t, Context, Error};
use chrono::{Duration, Utc};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use poise::CreateReply;
use std::collections::HashMap;
use uuid::Uuid;

/// Create an invite link
#[poise::command(slash_command, guild_only)]
pub async fn invites(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            eprintln!("Guild ID not found.");
            return Ok(()); // Exit if guild_id is not found
        }
    };

    let guild = match ctx.guild() {
        Some(g) => g.clone(),
        None => {
            eprintln!("Guild not found.");
            return Ok(()); // Handle the error appropriately
        }
    };

    let member = ctx.author_member().await.unwrap_or_default();
    let locale = ctx.data().config.get_guild_locale(&guild_id.to_string());

    // Validate guild configuration
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
            send_error_embed(ctx, locale, "commands.invites.errors.server_not_allowed").await?;
            return Ok(());
        }
    };

    // Validate member join date
    let min_stay_duration = Duration::seconds(
        guild_config
            .min_member_age
            .unwrap_or(ctx.data().config.bot.default_min_member_age) as i64,
    );

    if let Some(join_date) = member.joined_at {
        let join_date_utc = join_date.naive_utc().and_utc();
        let joined_time = Utc::now() - join_date_utc;
        if joined_time < min_stay_duration {
            let params =
                create_not_long_enough_params(min_stay_duration.num_days(), joined_time.num_days());
            send_not_long_enough_embed(ctx, locale, params).await?;
            return Ok(());
        }
    } else {
        send_error_embed(ctx, locale, "commands.invites.errors.join_date_not_found").await?;
        return Ok(());
    }

    // Validate member roles
    let role_with_limit = match member.roles.iter().find_map(|role_id| {
        guild_config
            .allowed_roles
            .iter()
            .find(|allowed_role| allowed_role.id == role_id.to_string())
    }) {
        Some(role) => role,
        None => {
            send_error_embed(ctx, locale, "commands.invites.errors.missing_permissions").await?;
            return Ok(());
        }
    };

    // Check invite usage limit
    let used_invites = crate::utils::db::count_used_invites(
        &ctx.data().db,
        &ctx.author().id.to_string(),
        &guild_id.to_string(),
        role_with_limit.invite_limit.days,
    )
    .await?;

    if used_invites >= role_with_limit.invite_limit.count as i64 {
        let params = create_limit_params(role_with_limit, used_invites);
        send_limit_reached_embed(ctx, locale, params).await?;
        return Ok(());
    }

    // Create and record invite
    let invite_id = Uuid::new_v4().to_string();
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

    let guild_name = guild.name.clone();
    send_success_embed(
        ctx,
        locale,
        guild_name,
        role_with_limit,
        used_invites,
        bot_invite_url,
        guild.icon_url(),
    )
    .await?;

    Ok(())
}

async fn send_error_embed(ctx: Context<'_>, locale: &str, error_key: &str) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title(t!(locale, format!("{}.title", error_key).as_str()))
        .description(t!(locale, format!("{}.description", error_key).as_str()))
        .color(0xFF3333)
        .footer(CreateEmbedFooter::new(t!(
            locale,
            format!("{}.footer", error_key).as_str()
        )));

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

async fn send_limit_reached_embed(
    ctx: Context<'_>,
    locale: &str,
    params: HashMap<&str, String>,
) -> Result<(), Error> {
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
            ),
        ))
        .color(0xFF3333)
        .footer(CreateEmbedFooter::new(t!(
            locale,
            "commands.invites.errors.limit_reached.footer"
        )));

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

async fn send_not_long_enough_embed(
    ctx: Context<'_>,
    locale: &str,
    params: HashMap<&str, String>,
) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title(t!(locale, "commands.invites.errors.not_long_enough.title"))
        .description(t!(
            locale,
            "commands.invites.errors.not_long_enough.description",
            params.clone()
        ))
        .color(0xFF3333)
        .footer(CreateEmbedFooter::new(t!(
            locale,
            "commands.invites.errors.not_long_enough.footer",
            params.clone()
        )));

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

async fn send_success_embed(
    ctx: Context<'_>,
    locale: &str,
    guild_name: String,
    role_with_limit: &AllowedRole,
    used_invites: i64,
    bot_invite_url: String,
    guild_icon_url: Option<String>,
) -> Result<(), Error> {
    let params = create_success_params(role_with_limit, used_invites, &guild_name);
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
            t!(locale, "commands.invites.success.used_remaining", params),
        ))
        .color(0x4CACEE)
        .thumbnail(guild_icon_url.unwrap_or_default())
        .footer(CreateEmbedFooter::new(t!(
            locale,
            "commands.invites.success.footer"
        )));

    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

fn create_limit_params(role_with_limit: &AllowedRole, used_invites: i64) -> HashMap<&str, String> {
    let mut params = HashMap::new();
    params.insert("count", role_with_limit.invite_limit.count.to_string());
    params.insert("days", role_with_limit.invite_limit.days.to_string());
    params.insert("used", used_invites.to_string());
    params.insert(
        "remaining",
        (role_with_limit.invite_limit.count as i64 - used_invites).to_string(),
    );
    params
}

fn create_not_long_enough_params<'a>(days: i64, remaining: i64) -> HashMap<&'a str, String> {
    let mut params = HashMap::new();
    params.insert("days", days.to_string());
    params.insert("remaining", remaining.to_string());
    params
}

fn create_success_params<'a>(
    role_with_limit: &'a AllowedRole,
    used_invites: i64,
    guild_name: &'a str,
) -> HashMap<&'a str, String> {
    let mut params = HashMap::new();
    params.insert("guild", guild_name.to_string());
    params.insert("count", role_with_limit.invite_limit.count.to_string());
    params.insert("days", role_with_limit.invite_limit.days.to_string());
    params.insert("used", used_invites.to_string());
    params.insert(
        "remaining",
        (role_with_limit.invite_limit.count as i64 - used_invites).to_string(),
    );
    params
}
