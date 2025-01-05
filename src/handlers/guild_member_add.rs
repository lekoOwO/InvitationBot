use crate::Data;
use log::debug;
use poise::serenity_prelude::{self as serenity};

pub async fn handle_guild_member_add(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    new_member: &serenity::Member,
    data: &Data,
) {
    debug!(
        "New member {} added to guild: {}.",
        new_member.user.id, guild_id
    );
    // Find if this guild is in the config
    let guild_config = data
        .config
        .guilds
        .allowed
        .iter()
        .find(|g| g.id == guild_id.to_string());
    if guild_config.is_none() {
        debug!("Guild {} not found in config, skipping...", guild_id);
        return;
    }

    debug!("Guild {} found in config, checking invites...", guild_id);
    let channel_id =
        serenity::ChannelId::new(guild_config.unwrap().invite_channel.parse().unwrap());

    // Get all invites from this server
    if let Ok(invites) = channel_id.invites(&ctx.http).await {
        for invite in invites
            .into_iter()
            .filter(|invite| invite.max_uses == 2 && invite.uses == 1)
        {
            debug!(
                "Invite {} found in database for guild {} member {}.",
                invite.code, guild_id, new_member.user.id
            );

            // Check if this is our invite code
            if let Ok(Some(invite_id)) =
                crate::utils::db::find_invite_by_code(&data.db, &invite.code).await
            {
                debug!(
                    "Invite {} found in database for guild {} member {}.",
                    invite_id, guild_id, new_member.user.id
                );
                // Record the user
                let _ = crate::utils::db::record_invite_use(
                    &data.db,
                    &invite_id,
                    &new_member.user.id.to_string(),
                )
                .await;

                // Delete the invite link
                let _ = invite.delete(&ctx.http).await;
                break;
            }
        }
    }
}
