use sqlx::SqlitePool;
use sqlx::types::time::OffsetDateTime;

pub async fn setup_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS invites (
            id TEXT PRIMARY KEY,
            guild_id TEXT NOT NULL,
            creator_id TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            used_at DATETIME,
            used_by TEXT,
            discord_invite_code TEXT
        )"
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_pool(database_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(&format!("sqlite:{}", database_path)).await?;
    setup_database(&pool).await?;
    Ok(pool)
}

// 邀請相關操作
pub async fn create_invite(
    pool: &SqlitePool,
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
    pool: &SqlitePool,
    invite_id: &str,
) -> Result<Option<InviteRecord>, sqlx::Error> {
    sqlx::query_as!(
        InviteRecord,
        "SELECT guild_id FROM invites WHERE id = ? AND used_at IS NULL",
        invite_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_invite_code(
    pool: &SqlitePool,
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
    pool: &SqlitePool,
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
pub struct InviteRecord {
    pub guild_id: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct InviteInfo {
    pub creator_id: Option<String>,
    pub used_at: Option<OffsetDateTime>,
    pub discord_invite_code: Option<String>,
}

pub async fn get_user_invite_info(
    pool: &SqlitePool,
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

// 新增：記錄使用邀請的用戶
pub async fn record_invite_use(
    pool: &SqlitePool,
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
    pool: &SqlitePool,
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
    pool: &SqlitePool,
    guild_id: &str,
    days: i32,
) -> Result<Vec<InviteLeaderboardEntry>, sqlx::Error> {
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
        ORDER BY invite_count DESC
        LIMIT 5
        "#,
        guild_id,
        days_str
    )
    .fetch_all(pool)
    .await
} 