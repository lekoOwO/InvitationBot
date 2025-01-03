use rust_embed::RustEmbed;

pub const AVAILABLE_LOCALES: [&str; 2] = ["en", "zh-TW"];

#[derive(RustEmbed)]
#[folder = "src/i18n"]
pub struct I18nAssets;

pub fn get_yaml(locale: &str) -> Option<String> {
    I18nAssets::get(&format!("{}.yaml", locale))
        .map(|f| String::from_utf8_lossy(f.data.as_ref()).into_owned())
} 