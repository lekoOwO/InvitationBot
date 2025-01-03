use std::collections::HashMap;
use std::fs;
use once_cell::sync::Lazy;
use serde_yaml::Value;

type Translations = HashMap<String, Value>;

static TRANSLATIONS: Lazy<HashMap<String, Translations>> = Lazy::new(|| {
    let mut translations = HashMap::new();
    
    // Load all language files
    for locale in &["en", "zh-TW"] {
        let path = format!("src/i18n/{}.yaml", locale);
        if let Ok(content) = fs::read_to_string(&path) {
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
    
    let translations = TRANSLATIONS.get(locale).or_else(|| TRANSLATIONS.get(default_locale));
    
    if let Some(trans) = translations {
        let mut value = trans.get(&parts[0].to_string())
            .unwrap_or_else(|| return &Value::Null);
        
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