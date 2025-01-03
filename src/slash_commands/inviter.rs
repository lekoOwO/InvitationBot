use crate::{t, Context, Error};
use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, User};
use poise::CreateReply;
use std::collections::HashMap;

/// View who invited a user
#[poise::command(slash_command, guild_only)]
pub async fn inviter(
    ctx: Context<'_>,
    #[description = "User to check"] user: User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let locale = ctx.data().config.get_guild_locale(&guild_id.to_string());

    let invite_info =
        match crate::utils::db::get_user_invite_info(&ctx.data().db, &user.id.to_string()).await? {
            Some(info) => info,
            None => {
                let mut params = HashMap::new();
                params.insert("user", format!("<@{}>", user.id));

                let embed = CreateEmbed::default()
                    .title(t!(locale, "commands.inviter.errors.no_record.title"))
                    .description(t!(
                        locale,
                        "commands.inviter.errors.no_record.description",
                        params
                    ))
                    .color(0xFF3333)
                    .footer(CreateEmbedFooter::new(t!(
                        locale,
                        "commands.inviter.errors.no_record.footer"
                    )));

                ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                    .await?;
                return Ok(());
            }
        };

    let creator = match invite_info.creator_id.unwrap().parse() {
        Ok(id) => ctx.http().get_user(id).await?,
        Err(_) => {
            let embed = CreateEmbed::default()
                .title(t!(locale, "commands.inviter.errors.invalid_user.title"))
                .description(t!(
                    locale,
                    "commands.inviter.errors.invalid_user.description"
                ))
                .color(0xFF3333)
                .footer(CreateEmbedFooter::new(t!(
                    locale,
                    "commands.inviter.errors.invalid_user.footer"
                )));

            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    let used_at =
        chrono::DateTime::<Utc>::from_timestamp(invite_info.used_at.unwrap().unix_timestamp(), 0)
            .unwrap();
    let time_ago = {
        let duration = Utc::now().signed_duration_since(used_at);
        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else {
            format!("{} minutes ago", duration.num_minutes())
        }
    };

    let mut params = HashMap::new();
    params.insert("user", format!("<@{}>", user.id));
    params.insert("inviter", format!("<@{}>", creator.id));
    params.insert("date", used_at.format("%Y-%m-%d %H:%M:%S").to_string());
    params.insert("time_ago", time_ago);
    params.insert("code", invite_info.discord_invite_code.unwrap_or_default());

    let embed = CreateEmbed::default()
        .title(t!(locale, "commands.inviter.success.title"))
        .description(format!(
            "**{}**: {}\n**{}**: {}\n**{}**: {} ({})\n**{}**: {}",
            t!(locale, "commands.inviter.success.user"),
            params["user"],
            t!(locale, "commands.inviter.success.invited_by"),
            params["inviter"],
            t!(locale, "commands.inviter.success.date"),
            params["date"],
            params["time_ago"],
            t!(locale, "commands.inviter.success.invite_code"),
            params["code"]
        ))
        .color(0x4CACEE)
        .thumbnail(user.avatar_url().unwrap_or_default())
        .footer(CreateEmbedFooter::new(t!(
            locale,
            "commands.inviter.success.footer"
        )));

    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
}
