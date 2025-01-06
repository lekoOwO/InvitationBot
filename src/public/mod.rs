use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/public"]
#[exclude = "mod.rs"]
pub struct Assets;
