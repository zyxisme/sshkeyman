use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static"]
pub struct StaticAssets;

#[derive(RustEmbed)]
#[folder = "locales"]
pub struct LocaleAssets;
