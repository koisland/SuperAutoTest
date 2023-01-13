pub type LRegex = lazy_regex::Lazy<lazy_regex::Regex>;

pub static RGX_ATK: &LRegex = regex!(r#"(\d+)\s+attack"#);
pub static RGX_HEALTH: &LRegex = regex!(r#"(\d+)\s+health"#);
pub static RGX_DMG: &LRegex = regex!(r#"(\d+)\sdamage"#);
pub static RGX_N_TRIGGERS: &LRegex = regex!(r#"Triggers\s(\d+)\stimes"#);
pub static RGX_SUMMON_ATK: &LRegex = regex!(r#"(\d+)/"#);
pub static RGX_SUMMON_HEALTH: &LRegex = regex!(r#"/(\d+)"#);
pub static RGX_END_OF_BATTLE: &LRegex = regex!(r#"end of battle"#);
