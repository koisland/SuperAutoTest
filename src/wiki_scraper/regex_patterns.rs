pub type LRegex = lazy_regex::Lazy<lazy_regex::Regex>;

// General
pub static RGX_ICON_NAME: &LRegex = regex!(r#"\{\{IconSAP\|(.*?)[\|\}]+.*?([\w\|]*=[\w\.]+)*"#);

// Food
pub static RGX_TABLE: &LRegex = regex!(r#"\{\|(.|\W)*\|\}"#);
pub static RGX_COLS: &LRegex = regex!(r#"!(.*?)\n"#);
pub static RGX_FOOD_LINK_NAME: &LRegex = regex!(r#"\[\[(.*?)\]\]"#);

// Pet
pub static RGX_TIER: &LRegex = regex!(r#"<!--\sTIER\s(\d)\s-->"#);
pub static RGX_PET_NAME: &LRegex = regex!(r#"pet\s=\s\{\{IconSAP\|(.*?)\|size"#);
pub static RGX_PET_STATS: &LRegex =
    regex!(r#"attack\s=\s(?P<attack>\d+)\s\|\shealth\s=\s(?P<health>\d+)"#);
pub static RGX_PET_PACK: &LRegex = regex!(r#"(\w+pack)+"#);
pub static RGX_PET_EFFECT_TRIGGER: &LRegex = regex!(r#"\| '''(.*?)'''+"#);
pub static RGX_PET_EFFECT: &LRegex = regex!(r#"→\s(.*?)\n"#);
pub static RGX_PET_EFFECT_TRIGGERLESS: &LRegex = regex!(r#"\|\s([^[=]]*?\.*)\n"#);

// Token
pub static RGX_TOKEN_SPAN_COLS: &LRegex = regex!(r#"!\s(col|row)span="(\d+)"\s\|(.*?)\n"#);
pub static RGX_SUMMON_STATS: &LRegex = regex!(r#"\|\scolspan="(\d+)"\s\|([\d\w]+/[\d\w]+)\n"#);
