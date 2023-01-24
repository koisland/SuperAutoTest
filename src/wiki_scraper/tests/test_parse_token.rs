use std::str::FromStr;

use crate::{
    db::{pack::Pack, record::PetRecord},
    wiki_scraper::{
        common::read_wiki_url,
        parse_food::get_largest_table,
        parse_tokens::{clean_token_block, parse_single_token, parse_token_info, TokenTableCols},
    },
};

const TOKEN_TABLE: &str = "
{| class=\"sortable fandom-table\"
! rowspan=\"2\" |Name
! colspan=\"3\" |[[File:AttackHealth.png|frameless|53x53px]]
! rowspan=\"2\" |Summoned From
! rowspan=\"2\" |Additional Notes
|-
!Level 1
!Level 2
!Level 3
|-
|{{IconSAP|Bee}}
| colspan=\"3\" |1/1
|Any [[Pets|Animal]] with {{IconSAP|Honey}} once it faints.
|
|}
";

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
fn test_parse_tokens() {
    let wikis = read_wiki_url(crate::SCRAPER_SOURCES).unwrap();
    assert!(parse_token_info(&wikis.tokens).is_ok())
}

#[test]
fn test_parse_single_token_explicit_lvl_stats() {
    let mut pets: Vec<PetRecord> = vec![];

    let cols = vec![
        TokenTableCols::Name,
        TokenTableCols::StatsLvl1,
        TokenTableCols::StatsLvl2,
        TokenTableCols::StatsLvl3,
        TokenTableCols::SummonedFrom,
        TokenTableCols::Notes,
    ];

    parse_single_token(TOKEN_BUS, &cols, &mut pets).unwrap();

    let exp_pets = [
        PetRecord {
            name: "Bus".to_string(),
            tier: 0,
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
        },
        PetRecord {
            name: "Bus".to_string(),
            tier: 0,
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
        },
        PetRecord {
            name: "Bus".to_string(),
            tier: 0,
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
        },
    ];
    assert_eq!(pets, exp_pets)
}
#[test]
fn test_parse_single_token_colspan() {
    let mut pets: Vec<PetRecord> = vec![];

    let cols = vec![
        TokenTableCols::Name,
        TokenTableCols::StatsLvl1,
        TokenTableCols::StatsLvl2,
        TokenTableCols::StatsLvl3,
        TokenTableCols::SummonedFrom,
        TokenTableCols::Notes,
    ];

    parse_single_token(TOKEN_BUTTERFLY, &cols, &mut pets).unwrap();

    let exp_pets = [
        PetRecord {
            name: "Butterfly".to_string(),
            tier: 0,
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
        },
        PetRecord {
            name: "Butterfly".to_string(),
            tier: 0,
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
        },
        PetRecord {
            name: "Butterfly".to_string(),
            tier: 0,
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
        },
    ];

    assert_eq!(pets, exp_pets)
}

#[test]
fn test_parse_single_token_invalid_col_num() {
    let mut pets: Vec<PetRecord> = vec![];

    let cols = vec![
        TokenTableCols::Name,
        TokenTableCols::StatsLvl1,
        TokenTableCols::StatsLvl2,
        TokenTableCols::SummonedFrom,
        TokenTableCols::Notes,
    ];

    assert!(parse_single_token(TOKEN_BUTTERFLY, &cols, &mut pets).is_err());
}

#[test]
fn test_parse_invalid_token_cols() {
    // Case sensitive match.
    assert!(TokenTableCols::from_str("name").is_err())
}

#[test]
fn test_parse_token_cols() {
    let table = get_largest_table(&TOKEN_TABLE).unwrap();
    let cols = TokenTableCols::get_cols(&table).unwrap();
    assert_eq!(
        cols,
        vec![
            TokenTableCols::Name,
            TokenTableCols::StatsLvl1,
            TokenTableCols::StatsLvl2,
            TokenTableCols::StatsLvl3,
            TokenTableCols::SummonedFrom,
            TokenTableCols::Notes
        ]
    )
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
