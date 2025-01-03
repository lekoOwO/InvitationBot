use once_cell::sync::Lazy;
use serde_yaml::Value;
use std::collections::HashMap;

type Translations = HashMap<String, Value>;

static TRANSLATIONS: Lazy<HashMap<String, Translations>> = Lazy::new(|| {
    let mut translations = HashMap::new();

    // Load all language files from embedded assets
    for locale in crate::i18n::AVAILABLE_LOCALES.iter() {
        if let Some(content) = crate::i18n::get_yaml(locale) {
            if let Ok(trans) = serde_yaml::from_str(&content) {
                translations.insert(locale.to_string(), trans);
            }
        }
    }

    translations
});

pub fn get_text(locale: &str, key: &str, params: Option<HashMap<&str, String>>) -> String {
    let parts: Vec<&str> = key.split('.').collect();
    let default_locale = "en";

    let translations = TRANSLATIONS
        .get(locale)
        .or_else(|| TRANSLATIONS.get(default_locale));

    if let Some(trans) = translations {
        let mut value = trans.get(&parts[0].to_string()).unwrap_or_else(|| {
            println!("Translation not found for key: {}", key);
            &Value::Null
        });

        for &part in &parts[1..] {
            if let Some(v) = value.get(part) {
                value = v;
            } else {
                return key.to_string();
            }
        }

        if let Some(text) = value.as_str() {
            if let Some(params) = &params {
                let mut result = text.to_string();
                for (key, value) in params {
                    result = result.replace(&format!("{{{}}}", key), value);
                }
                return result;
            }
            return text.to_string();
        }
    }

    key.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_text() {
        // Test simple text
        assert_eq!(get_text("en", "commands.ping.name", None), "ping");

        // Test with parameters
        let mut params = HashMap::new();
        params.insert("name", "TestBot".to_string());
        assert_eq!(
            get_text("en", "bot.logged_in", Some(params)),
            "Logged in as TestBot"
        );

        // Test fallback to default locale
        assert_eq!(get_text("invalid", "commands.ping.name", None), "ping");

        // Test invalid key
        assert_eq!(get_text("en", "invalid.key", None), "invalid.key");
    }

    #[test]
    fn test_translations_loaded() {
        for locale in crate::i18n::AVAILABLE_LOCALES.iter() {
            assert!(
                TRANSLATIONS.get(*locale).is_some(),
                "Missing translations for locale: {}",
                locale
            );
        }
    }
}
