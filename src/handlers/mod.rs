use poise::serenity_prelude::{self as serenity};
use crate::Data;

pub async fn handle_guild_member_addition(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    new_member: &serenity::Member,
    data: &Data,
) {
    // Get all invites from this server
    if let Ok(invites) = guild_id.invites(&ctx.http).await {
        for invite in invites {
            // Check if this is our invite code
            if let Ok(Some(invite_id)) = crate::utils::db::find_invite_by_code(&data.db, &invite.code).await {
                // Check if this invite is set to 2 uses, has been used once, and was created by us
                // If these conditions are met, we can assume this invite was used by the new member
                if invite.max_uses == 2 && invite.uses == 1 {
                    // Record the user
                    let _ = crate::utils::db::record_invite_use(
                        &data.db,
                        &invite_id,
                        &new_member.user.id.to_string(),
                    ).await;
                    
                    // Delete the invite link
                    let _ = invite.delete(&ctx.http).await;
                    break;
                }
            }
        }
    }
} 