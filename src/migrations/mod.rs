use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/migrations"]
#[include = "*.sql"]
pub struct Migrations;

pub fn get_migration(name: &str) -> Option<String> {
    Migrations::get(name).map(|f| String::from_utf8_lossy(f.data.as_ref()).into_owned())
}

pub fn get_migrations() -> Vec<String> {
    Migrations::iter().map(|f| f.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_migration() {
        // Test existing migration
        let migration =
            get_migration("20240101000000_create_tables.sql").expect("Failed to load migration");
        assert!(migration.contains("CREATE TABLE IF NOT EXISTS invites"));

        // Test non-existing migration
        assert!(get_migration("invalid.sql").is_none());
    }

    #[test]
    fn test_get_migrations() {
        let migrations = get_migrations();
        assert!(!migrations.is_empty());
        assert!(migrations.contains(&"20240101000000_create_tables.sql".to_string()));
    }
}
