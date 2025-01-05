use sqlx::types::time::OffsetDateTime;
type Pool = sqlx::Pool<sqlx::Sqlite>;

pub async fn setup_database(pool: &Pool) -> Result<(), sqlx::Error> {
    let migrations = crate::migrations::get_migrations();
    for migration in migrations {
        if let Some(sql) = crate::migrations::get_migration(migration.as_str()) {
            sqlx::query(&sql).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn create_pool(database_path: &str) -> Result<Pool, sqlx::Error> {
    let pool = Pool::connect(database_path).await?;
    setup_database(&pool).await?;
    Ok(pool)
}

pub async fn create_invite(
    pool: &Pool,
    invite_id: &str,
    guild_id: &str,
    creator_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO invites (
            id, guild_id, creator_id, created_at
        ) VALUES (?, ?, ?, datetime('now'))",
        invite_id,
        guild_id,
        creator_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_unused_invite(
    pool: &Pool,
    invite_id: &str,
) -> Result<Option<InviteRecord>, sqlx::Error> {
    sqlx::query_as!(
        InviteRecord,
        "SELECT guild_id, creator_id, discord_invite_code as code FROM invites WHERE id = ? AND used_at IS NULL",
        invite_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_invite_code(
    pool: &Pool,
    invite_id: &str,
    discord_code: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE invites SET discord_invite_code = ? WHERE id = ?",
        discord_code,
        invite_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_used_invites(
    pool: &Pool,
    creator_id: &str,
    guild_id: &str,
    days: i32,
) -> Result<i64, sqlx::Error> {
    let days_str = format!("-{} days", days);
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) as count 
         FROM invites 
         WHERE creator_id = ? 
         AND created_at > datetime('now', ?) 
         AND guild_id = ?
         AND used_at IS NOT NULL",
        creator_id,
        days_str,
        guild_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count)
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct InviteRecord {
    pub guild_id: String,
    pub creator_id: String,   // Used for invite tracking and permissions
    pub code: Option<String>, // Discord invite code, if already created
}

#[derive(Debug, sqlx::FromRow)]
pub struct InviteInfo {
    pub creator_id: Option<String>,
    pub used_at: Option<OffsetDateTime>,
    pub discord_invite_code: Option<String>,
}

pub async fn get_user_invite_info(
    pool: &Pool,
    user_id: &str,
) -> Result<Option<InviteInfo>, sqlx::Error> {
    sqlx::query_as!(
        InviteInfo,
        "SELECT creator_id, used_at, discord_invite_code
         FROM invites 
         WHERE used_by = ? 
         AND used_at IS NOT NULL
         ORDER BY used_at DESC
         LIMIT 1",
        user_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn record_invite_use(
    pool: &Pool,
    invite_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE invites 
        SET used_at = datetime('now'), 
            used_by = ? 
        WHERE id = ? 
        AND used_at IS NULL
        "#,
        user_id,
        invite_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_invite_by_code(
    pool: &Pool,
    discord_code: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM invites WHERE discord_invite_code = ? AND used_at IS NULL LIMIT 1",
        discord_code
    )
    .fetch_optional(pool)
    .await
    .map(|opt| opt.flatten())
}

#[derive(Debug, sqlx::FromRow)]
pub struct InviteLeaderboardEntry {
    pub creator_id: String,
    pub invite_count: i64,
}

pub async fn get_invite_leaderboard(
    pool: &Pool,
    guild_id: &str,
    days: i32,
) -> Result<Vec<InviteLeaderboardEntry>, sqlx::Error> {
    // Ensure days is non-negative
    let days = days.max(0);
    let days_str = format!("-{} days", days);

    sqlx::query_as!(
        InviteLeaderboardEntry,
        r#"
        SELECT 
            creator_id,
            COUNT(*) as invite_count
        FROM invites 
        WHERE guild_id = ?
        AND created_at > datetime('now', ?)
        AND used_at IS NOT NULL
        GROUP BY creator_id
        ORDER BY invite_count DESC, creator_id ASC
        LIMIT 5
        "#,
        guild_id,
        days_str
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    async fn setup_test_db() -> Pool {
        let db_url = format!("sqlite:file:{}?mode=memory", Uuid::new_v4());
        let pool = create_pool(&db_url).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_invite() {
        let pool = setup_test_db().await;
        let invite_id = Uuid::new_v4().to_string();
        let guild_id = "123456789";
        let creator_id = "987654321";

        // Test create invite
        create_invite(&pool, &invite_id, guild_id, creator_id)
            .await
            .unwrap();

        // Test get unused invite
        let invite = get_unused_invite(&pool, &invite_id).await.unwrap().unwrap();
        assert_eq!(invite.guild_id, guild_id);
        assert_eq!(invite.creator_id, creator_id);
        assert!(invite.code.is_none());
    }

    #[tokio::test]
    async fn test_invite_leaderboard() {
        let pool = setup_test_db().await;
        let guild_id = "123456789";
        let creator_id = "987654321";
        let user_id = "111222333";

        // Create multiple invites
        for _ in 0..3 {
            let invite_id = Uuid::new_v4().to_string();
            create_invite(&pool, &invite_id, guild_id, creator_id)
                .await
                .unwrap();
            record_invite_use(&pool, &invite_id, user_id).await.unwrap();
        }

        // Test leaderboard
        let entries = get_invite_leaderboard(&pool, guild_id, 30).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].creator_id, creator_id);
        assert_eq!(entries[0].invite_count, 3);
    }

    #[tokio::test]
    async fn test_mark_invite_used() {
        let pool = setup_test_db().await;
        let invite_id = Uuid::new_v4().to_string();
        let guild_id = "123456789";
        let creator_id = "987654321";
        let user_id = "111222333";

        // Create invite
        create_invite(&pool, &invite_id, guild_id, creator_id)
            .await
            .unwrap();

        // Mark as used
        record_invite_use(&pool, &invite_id, user_id).await.unwrap();

        // Verify invite is marked as used
        let invite = get_unused_invite(&pool, &invite_id).await.unwrap();
        assert!(invite.is_none());
    }
}
