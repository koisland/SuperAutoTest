pub type LRegex = lazy_regex::Lazy<lazy_regex::Regex>;
#[allow(dead_code)]
pub static RGX_PERC: &LRegex = regex!(r#"(\d+)%"#);
pub static RGX_ATK: &LRegex = regex!(r#"(\d)\sattack"#);
pub static RGX_HEALTH: &LRegex = regex!(r#"(\d)\shealth"#);
#[allow(dead_code)]
pub static RGX_DMG: &LRegex = regex!(r#"(\d+)\sdamage"#);
pub static RGX_N_TRIGGERS: &LRegex = regex!(r#"Triggers\s(\d+)\stimes"#);
pub static RGX_SUMMON_ATK: &LRegex = regex!(r#"(\d+)/"#);
pub static RGX_SUMMON_HEALTH: &LRegex = regex!(r#"/(\d+)"#);
