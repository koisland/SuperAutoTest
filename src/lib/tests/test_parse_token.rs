use crate::{
    db::{pack::Pack, record::PetRecord},
    wiki_scraper::parse_tokens::{clean_token_block, parse_single_token},
    PetName,
};

const TOKEN_BUS: &str = "
|{{IconSAP|Bus}}
|5/5
|10/10
|15/15
| A {{IconSAP|Deer}} once it faints.
|Bus has innate {{IconSAP|Chili}}
";

const TOKEN_BUTTERFLY: &str = "
|{{IconSAP|Butterfly}}
| colspan=\"3\" |1/1
|A level 3 {{IconSAP|Caterpillar}} at the start of battle.
|Immediately copies the strength and health of the strongest unit.
";

#[test]
fn test_parse_single_token_explicit_lvl_stats() {
    let mut pets: Vec<PetRecord> = vec![];

    parse_single_token(TOKEN_BUS, &mut pets).unwrap();

    let exp_pets = [
        PetRecord {
            name: PetName::Bus,
            tier: 1,
            attack: 5,
            health: 5,
            pack: Pack::Unknown,
            effect_trigger: Some("A Deer once it faints.".to_string()),
            effect: Some("Bus has innate Chili".to_string()),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 0,
            temp_effect: false,
            lvl: 1,
            cost: 0,
            img_url: String::from("https://superautopets.wiki.gg/images/f/f0/Bus_Icon.png"),
            is_token: true,
        },
        PetRecord {
            name: PetName::Bus,
            tier: 1,
            attack: 10,
            health: 10,
            pack: Pack::Unknown,
            effect_trigger: Some("A Deer once it faints.".to_string()),
            effect: Some("Bus has innate Chili".to_string()),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 0,
            temp_effect: false,
            lvl: 2,
            cost: 0,
            img_url: String::from("https://superautopets.wiki.gg/images/f/f0/Bus_Icon.png"),
            is_token: true,
        },
        PetRecord {
            name: PetName::Bus,
            tier: 1,
            attack: 15,
            health: 15,
            pack: Pack::Unknown,
            effect_trigger: Some("A Deer once it faints.".to_string()),
            effect: Some("Bus has innate Chili".to_string()),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 0,
            temp_effect: false,
            lvl: 3,
            cost: 0,
            img_url: String::from("https://superautopets.wiki.gg/images/f/f0/Bus_Icon.png"),
            is_token: true,
        },
    ];
    assert_eq!(pets, exp_pets)
}

#[test]
fn test_parse_single_token_colspan() {
    let mut pets: Vec<PetRecord> = vec![];

    parse_single_token(TOKEN_BUTTERFLY, &mut pets).unwrap();

    let exp_pets = [
        PetRecord {
            name: PetName::Butterfly,
            tier: 1,
            attack: 1,
            health: 1,
            pack: Pack::Unknown,
            effect_trigger: Some("A level 3 Caterpillar at the start of battle.".to_string()),
            effect: Some(
                "Immediately copies the strength and health of the strongest unit.".to_string(),
            ),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 0,
            temp_effect: false,
            lvl: 1,
            cost: 0,
            img_url: String::from("https://superautopets.wiki.gg/images/3/3d/Butterfly_Icon.png"),
            is_token: true,
        },
        PetRecord {
            name: PetName::Butterfly,
            tier: 1,
            attack: 1,
            health: 1,
            pack: Pack::Unknown,
            effect_trigger: Some("A level 3 Caterpillar at the start of battle.".to_string()),
            effect: Some(
                "Immediately copies the strength and health of the strongest unit.".to_string(),
            ),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 0,
            temp_effect: false,
            lvl: 2,
            cost: 0,
            img_url: String::from("https://superautopets.wiki.gg/images/3/3d/Butterfly_Icon.png"),
            is_token: true,
        },
        PetRecord {
            name: PetName::Butterfly,
            tier: 1,
            attack: 1,
            health: 1,
            pack: Pack::Unknown,
            effect_trigger: Some("A level 3 Caterpillar at the start of battle.".to_string()),
            effect: Some(
                "Immediately copies the strength and health of the strongest unit.".to_string(),
            ),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 0,
            temp_effect: false,
            lvl: 3,
            cost: 0,
            img_url: String::from("https://superautopets.wiki.gg/images/3/3d/Butterfly_Icon.png"),
            is_token: true,
        },
    ];

    assert_eq!(pets, exp_pets)
}

#[test]
fn test_clean_token_block() {
    let cleaned_token = clean_token_block(TOKEN_BUTTERFLY).unwrap();

    let exp_cleaned_token = "Butterfly
|1/1
|1/1
|1/1
|A level 3 Caterpillar at the start of battle.
|Immediately copies the strength and health of the strongest unit.
";

    assert_eq!(cleaned_token, exp_cleaned_token);
}
