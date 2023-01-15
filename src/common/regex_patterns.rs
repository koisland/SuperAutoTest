pub type LRegex = lazy_regex::Lazy<lazy_regex::Regex>;

pub static RGX_ATK: &LRegex = regex!(r#"([\d-]+)\s+attack"#);
pub static RGX_HEALTH: &LRegex = regex!(r#"([\d-]+)\s+health"#);
pub static RGX_DMG: &LRegex = regex!(r#"(\d+)\sdamage"#);
pub static RGX_DMG_REDUCE: &LRegex = regex!(r#"(\d+)\sless\sdamage"#);
pub static RGX_START_TURN: &LRegex = regex!(r#"start\sof\severy\sturn"#);
pub static RGX_END_TURN: &LRegex = regex!(r#"end\sof\sturn"#);
pub static RGX_ONE_USE: &LRegex = regex!(r#"once"#);
pub static RGX_RANDOM: &LRegex = regex!(r#"(a|one|two|three|1|2|3)\srandom"#);
pub static RGX_N_TRIGGERS: &LRegex = regex!(r#"Triggers\s(\d+)\stimes"#);
pub static RGX_SUMMON_ATK: &LRegex = regex!(r#"(\d+)/"#);
pub static RGX_SUMMON_HEALTH: &LRegex = regex!(r#"/(\d+)"#);
pub static RGX_END_OF_BATTLE: &LRegex = regex!(r#"end of battle"#);
