pub type LRegex = lazy_regex::Lazy<lazy_regex::Regex>;

// General
pub static RGX_COL_DESC_CATEG: &LRegex = regex!(r#"\|\+\w+"#);
pub static RGX_COL_WORD: &LRegex = regex!(r#"\|([A-Za-z][A-Za-z ]+[A-Za-z])"#);
pub static RGX_ICON_NAME: &LRegex = regex!(r#"\{\{IconSAP\|(.*?)[\|\}]+.*?([\w\|]*=[\w\.]+)*"#);
pub static RGX_MULT_TABLE: &LRegex = regex!(r#"\{\|(.|\W)*?\|\}"#);

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

// Stats
pub static RGX_ATK: &LRegex = regex!(r#"([\d-]+)%*\s+attack"#);
pub static RGX_HEALTH: &LRegex = regex!(r#"([\d-]+)%*\s+health"#);
pub static RGX_ATK_HEALTH: &LRegex = regex!(r#"([\d-]+)%*\s+attack\sand\shealth"#);
pub static RGX_DMG: &LRegex = regex!(r#"(\d+)%*\sdamage"#);
pub static RGX_DMG_REDUCE: &LRegex = regex!(r#"(\d+)%*\sless\sdamage"#);

// Turn
pub static RGX_START_TURN: &LRegex = regex!(r#"start\sof\severy\sturn"#);
pub static RGX_END_TURN: &LRegex = regex!(r#"end\sof\sturn"#);
pub static RGX_ONE_USE: &LRegex = regex!(r#"once"#);
pub static RGX_RANDOM: &LRegex = regex!(r#"(a|one|two|three|1|2|3)\srandom"#);
pub static RGX_N_TRIGGERS: &LRegex = regex!(r#"(\d+)\stimes*"#);
pub static RGX_SUMMON_ATK: &LRegex = regex!(r#"(\d+)/"#);
pub static RGX_SUMMON_HEALTH: &LRegex = regex!(r#"/(\d+)"#);
pub static RGX_END_OF_BATTLE: &LRegex = regex!(r#"end of battle"#);
