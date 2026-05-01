use std::collections::HashMap;
use std::sync::LazyLock;

pub type LocaleMap = HashMap<&'static str, &'static str>;

static EN: LazyLock<LocaleMap> = LazyLock::new(|| load_locale("en"));
static ZH_CN: LazyLock<LocaleMap> = LazyLock::new(|| load_locale("zh-CN"));

fn load_locale(name: &str) -> LocaleMap {
    let content = match std::fs::read_to_string(format!("locales/{}.toml", name)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("i18n: failed to read locales/{}.toml: {}", name, e);
            return HashMap::new();
        }
    };

    let toml: toml::Table = match toml::from_str(&content) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("i18n: failed to parse locales/{}.toml: {}", name, e);
            return HashMap::new();
        }
    };

    let mut map = HashMap::new();
    flatten_toml(&toml, "", &mut map);
    map
}

fn flatten_toml(table: &toml::Table, prefix: &str, map: &mut HashMap<&'static str, &'static str>) {
    for (key, value) in table {
        match value {
            toml::Value::Table(t) => {
                let new_prefix = if prefix.is_empty() {
                    format!("{}_", key)
                } else {
                    format!("{}{}_", prefix, key)
                };
                flatten_toml(t, &new_prefix, map);
            }
            toml::Value::String(s) => {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}{}", prefix, key)
                };
                // Leak key and value for 'static lifetime
                map.insert(
                    Box::leak(full_key.into_boxed_str()),
                    Box::leak(s.clone().into_boxed_str()),
                );
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Locale {
    pub map: &'static LocaleMap,
    pub en: &'static LocaleMap,
}

impl Locale {
    /// Parse Accept-Language header and return the best matching Locale.
    /// Defaults to zh-CN when no match is found.
    pub fn from_accept_language(header: Option<&str>) -> Self {
        let en = &*EN;
        let zh_cn = &*ZH_CN;

        let map = match header {
            Some(h) => {
                let best = best_locale(h);
                match best.as_str() {
                    "zh-CN" => zh_cn,
                    "en" => en,
                    _ => zh_cn, // default
                }
            }
            None => zh_cn,
        };

        Locale { map, en }
    }

    /// Look up a translation key. Falls back to English, then to the key itself.
    pub fn t(&self, key: &str) -> &'static str {
        self.map
            .get(key)
            .copied()
            .or_else(|| self.en.get(key).copied())
            .unwrap_or_else(|| Box::leak(key.to_string().into_boxed_str()))
    }

    /// Resolve a flash message. If `raw` is a known translation key, translate it
    /// and substitute `{0}` with `param`. Otherwise, return `raw` as-is (backward
    /// compatibility with backend error strings).
    pub fn resolve_flash(&self, raw: &str, param: Option<&str>) -> String {
        let translated = self.t(raw);
        // If translation returned the raw string unchanged, it's not a key — passthrough
        if translated == raw {
            raw.replace('+', " ")
        } else if let Some(p) = param {
            translated.replace("{0}", p)
        } else {
            translated.to_string()
        }
    }
}

/// Parse Accept-Language header and return the best matching supported locale.
/// Supported: "zh-CN", "en". Examples:
///   "zh-CN,zh;q=0.9,en;q=0.8" -> "zh-CN"
///   "en-US,en;q=0.9" -> "en"
///   "ja-JP" -> "zh-CN" (default)
fn best_locale(header: &str) -> String {
    let mut best_q = 0.0f32;
    let mut best_lang = String::from("zh-CN");

    for part in header.split(',') {
        let part = part.trim();
        let (lang_tag, q) = if let Some((tag, q_part)) = part.split_once(';') {
            let q = q_part
                .trim()
                .strip_prefix("q=")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(1.0);
            (tag.trim(), q)
        } else {
            (part, 1.0)
        };

        if q <= best_q {
            continue;
        }

        let lang_lower = lang_tag.to_lowercase();
        if lang_lower.starts_with("zh") {
            best_q = q;
            best_lang = String::from("zh-CN");
        } else if lang_lower.starts_with("en") {
            best_q = q;
            best_lang = String::from("en");
        }
    }

    best_lang
}

/// Ensure locale files are loaded (call once at startup).
pub fn init() {
    LazyLock::force(&EN);
    LazyLock::force(&ZH_CN);
}
