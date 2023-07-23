pub type LRegex = lazy_regex::Lazy<lazy_regex::Regex>;

// General
pub static RGX_COL_DESC_CATEG: &LRegex = regex!(r#"\|\+\w+"#);
pub static RGX_COL_WORD: &LRegex = regex!(r#"\|([A-Za-z][A-Za-z ]+[A-Za-z])"#);
pub static RGX_ICON_NAME: &LRegex = regex!(r#"\{\{IconSAP\|(.*?)[\|\}]+.*?([\w\|]*=[\w\.]+)*"#);
pub static RGX_MULT_TABLE: &LRegex = regex!(r#"\{\|(.|\W)*?\|\}"#);
pub static RGX_TABLE: &LRegex = regex!(r#"\{\|(.|\W)*\|\}"#);
pub static RGX_TIER_TABLE: &LRegex =
    regex!(r#"\|-\|Tier (\d)+(?:\w|\W|\d)*?(\{\| class="sortable fandom-table"[\w\W]*?\|\})"#);
pub static RGX_TIER: &LRegex = regex!(r#"<!--\s*TIER\s*(\d)\s-->"#);
pub static RGX_LINK_NAME: &LRegex = regex!(r#"\[\[(.*?)\]\]"#);

// Food
// TODO: This breaks if end of row template curly braces on same line as ability.
pub static RGX_FOOD_ROW: &LRegex = regex!(r#"\{\{:Foods/row([\s\W|\w]*?)\n\}\}"#);
pub static RGX_FOOD_NAME: &LRegex = regex!(r#"food\s*=\s*(.*?)\n"#);
pub static RGX_FOOD_EFFECT: &LRegex = regex!(r#"ability\s*=\s*([\s\W|\w]*)"#);

// Toy
// Reuses some regex from Pet that follows same syntax.
pub static RGX_TOY_ROW: &LRegex = regex!(r#"\{\{:Toys/row([\s\W|\w]*?)\n\}\}"#);
pub static RGX_TOY_NAME: &LRegex = regex!(r#"name\s*=\s*(.*?)\n"#);
pub static RGX_TOY_SOURCE: &LRegex = regex!(r#"source\s*=\s*(.*?)\n"#);

// Pet
pub static RGX_PET_ROW: &LRegex = regex!(r#"\{\{:Pets/row([\s\W|\w]*?)\n\}\}"#);
pub static RGX_PET_NAME: &LRegex = regex!(r#"pet\s*=\s*(.*?)\n"#);
pub static RGX_PET_STATS: &LRegex =
    regex!(r#"attack\s*=\s*(?P<attack>\d+)\s*\|\s*health\s*=\s*(?P<health>\d+)"#);
pub static RGX_PET_PACK: &LRegex = regex!(r#"(\w+pack)+"#);
pub static RGX_PET_EFFECT_TRIGGER: &LRegex = regex!(r#"\|\s*'''(.*?)'''"#);
pub static RGX_PET_EFFECT: &LRegex = regex!(r#"→\s*(.*?)\n"#);
pub static RGX_PET_EFFECT_TRIGGERLESS: &LRegex = regex!(r#"\|\s*([^[=]]*?\.*)\n"#);

// Token
pub static RGX_TOKEN_SPAN_COLS: &LRegex = regex!(r#"!\s*(col|row)span="(\d+)"\s*\|(.*?)\n"#);
pub static RGX_SUMMON_STATS: &LRegex = regex!(r#"\|\s*colspan="(\d+)"\s*\|([\d\w]+/[\d\w]+)\n"#);

// Stats
pub static RGX_ATK: &LRegex = regex!(r#"([\d-]+)%*\s+attack|\[\[File:Attack"#);
pub static RGX_HEALTH: &LRegex = regex!(r#"([\d-]+)%*\s+health|\[\[File:Health"#);
pub static RGX_ATK_HEALTH: &LRegex = regex!(r#"([\d-]+)%*\s+attack\sand\shealth"#);
pub static RGX_DMG: &LRegex = regex!(r#"(\d+)%*\s*damage"#);
pub static RGX_DMG_REDUCE: &LRegex = regex!(r#"(\d+)%*\s*less\sdamage"#);

// Turn
pub static RGX_START_TURN: &LRegex = regex!(r#"start\sof\severy\sturn"#);
pub static RGX_END_TURN: &LRegex = regex!(r#"end\sof\sturn"#);
pub static RGX_ONE_USE: &LRegex = regex!(r#"once"#);
pub static RGX_RANDOM: &LRegex = regex!(r#"(a|one|two|three|1|2|3)\srandom"#);
pub static RGX_N_TRIGGERS: &LRegex = regex!(r#"(\d+)\stimes*"#);
pub static RGX_SUMMON_ATK: &LRegex = regex!(r#"(\d+)/"#);
pub static RGX_SUMMON_HEALTH: &LRegex = regex!(r#"/(\d+)"#);
pub static RGX_END_OF_BATTLE: &LRegex = regex!(r#"end of battle"#);
