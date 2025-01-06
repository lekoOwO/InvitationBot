use rust_embed::RustEmbed;

pub const AVAILABLE_LOCALES: [&str; 2] = ["en", "zh-TW"];

#[derive(RustEmbed)]
#[folder = "src/i18n"]
#[include = "*.yaml"]
pub struct I18nAssets;

pub fn get_yaml(locale: &str) -> Option<String> {
    I18nAssets::get(&format!("{}.yaml", locale))
        .map(|f| String::from_utf8_lossy(f.data.as_ref()).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_yaml() {
        // Test existing locale
        let content = get_yaml("en").expect("Failed to load en.yaml");
        assert!(content.contains("commands"));

        // Test non-existing locale
        assert!(get_yaml("invalid").is_none());
    }

    #[test]
    fn test_available_locales() {
        for locale in AVAILABLE_LOCALES.iter() {
            assert!(
                get_yaml(locale).is_some(),
                "Missing yaml file for locale: {}",
                locale
            );
        }
    }
}
