use itertools::Itertools;

use crate::{
    db::{pack::Pack, record::FoodRecord},
    wiki_scraper::parse_food::{
        clean_link_text, get_effect_attack, get_effect_health, get_food_cost, get_largest_table,
        get_random_n_effect, is_holdable_item, is_temp_single_use, is_turn_effect,
        parse_one_food_entry, FoodTableCols,
    },
    FoodName,
};

const PAGE_INFO: &str = "
{| class=\"Test table\"
!Hello world!
|-
| Goodbye!
|}

Foods in Pack 2 and not in any pack are subject to change.
{| class=\"sortable fandom-table\"
!Tier
!Name
!Effect
!Turtle Pack
!Puppy Pack
!Star Pack
|-
|1
|{{IconSAP|Apple}}
|Give one [[Pets|pet]] +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}}.
|Yes
|Yes
|Yes (summoned)
|-
|N/A
|{{IconSAP|Peanuts}}
|Knockout any pet attacked and hurt by this.
|Yes (summoned)
|Yes
(summoned)
|Yes (summoned)
|}
";

const STEAK_ENTRY: &str = "
|6
|{{IconSAP|Steak}}
|Give one [[Pets|pet]] [[Steak]].
Attack with +20 damage, once.
|Yes
|Yes
|No
";

const CUPCAKE_ENTRY: &str = "
|2
|{{IconSAP|Cupcake}}
|Give one [[Pets|pet]] +3 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}} until end of battle.
|Yes
|Yes
|No
";

const CUPCAKE_ENTRY_NEW_PACK: &str = "
|2
|{{IconSAP|Cupcake}}
|Give one [[Pets|pet]] +3 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}} until end of battle.
|Yes
|Yes
|No
|No
";

const CUPCAKE_ENTRY_MISSING_FIELD: &str = "
|2
|{{IconSAP|Cupcake}}
|Give one [[Pets|pet]] +3 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}} until end of battle.
|Yes
|Yes
";
const BROCCOLI_ENTRY: &str = "
|2
|{{IconSAP|Broccoli}}
|Give one pet -1 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}}.
|No
|No
|Yes
";

const SHRIMP_ENTRY: &str = "
|2
|{{IconSAP|Fried Shrimp}}
|Give one pet +3 {{IconSAP|attack|nolink=yes}} and -1 {{IconSAP|health|nolink=yes}}.
|No
|No
|Yes
";

const GARLIC_ENTRY: &str = "
|3
|{{IconSAP|Garlic}}
|Give one [[Pets|Pet]] [[Garlic]].
Take 2 less damage.
|Yes
|Yes
|No
";

const GRAPES_ENTRY: &str = "
|4
|{{IconSAP|Grapes}}
|Give one [[Pets|pet]] [[Grapes]].
Gain +1 {{IconSAP|Gold}} at the start of every turn.
|No
|No
|Yes
";

const CARROT_ENTRY: &str = "
|5
|{{IconSAP|Carrot}}
|Give one [[Pets|pet]] [[Carrot]].
Gain +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}} at end of turn.
|No
|No
|Yes
";

const SUSHI_ENTRY: &str = "
|5
|{{IconSAP|Sushi}}
|Give three random [[pets]] +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}}.
|Yes
|Yes
|No
";

const PEANUTS_ENTRY: &str = "
|N/A
|{{IconSAP|Peanut}}
|Knockout any pet attacked and hurt by this.
|Yes (summoned)
|Yes
(summoned)
|Yes (summoned)
";

const SLEEPING_PILL_ENTRY: &str = "
|2
|{{IconSAP|Sleeping Pill}}
|Make one pet faint. Always on sale!
|Yes
|Yes
|No
";

const CHILI_ENTRY: &str = "
|5
|{{IconSAP|Chili}}
|Give one [[Pets|pet]] [[Chili]].
Attack second enemy for 5 {{IconSAP|damage|nolink=yes}}.
|Yes
|Yes
|No
";

const HONEY_ENTRY: &str = "
|1
|{{IconSAP|Honey}}
|Give one [[Pets|pet]] [[Honey]].
Summon a 1/1 {{IconSAP|Bee}} after fainting.
|Yes
|Yes
|No
";

const POPCORNS_ENTRY: &str = "
|6
|{{IconSAP|Popcorns}}
|Give one [[Pets|pet]] [[Popcorns]].
Summon a random pet from the same tier after fainting.
|No
|No
|Yes
";

#[test]
fn test_get_table() {
    let res = get_largest_table(PAGE_INFO).unwrap();
    // Cols, table_1, table_2, ...
    let exp_res = [
        "{| class=\"sortable fandom-table\"\n!Tier\n!Name\n!Effect\n!Turtle Pack\n!Puppy Pack\n!Star Pack\n",
        "\n|1\n|{{IconSAP|Apple}}\n|Give one [[Pets|pet]] +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}}.\n|Yes\n|Yes\n|Yes (summoned)\n",
        "\n|N/A\n|{{IconSAP|Peanuts}}\n|Knockout any pet attacked and hurt by this.\n|Yes (summoned)\n|Yes\n(summoned)\n|Yes (summoned)\n|}"
    ];

    assert_eq!(res, exp_res)
}

#[test]
fn test_get_table_cols() {
    let table = get_largest_table(PAGE_INFO).unwrap();

    let cols = FoodTableCols::get_cols(table.first().unwrap()).unwrap();
    let exp_cols = vec![
        FoodTableCols::Tier,
        FoodTableCols::Name,
        FoodTableCols::Effect,
        FoodTableCols::GamePack(Pack::Turtle),
        FoodTableCols::GamePack(Pack::Puppy),
        FoodTableCols::GamePack(Pack::Star),
    ];
    for (col, exp_col) in cols.iter().zip_eq(&exp_cols) {
        assert_eq!(col, exp_col)
    }
}

#[test]
fn test_clean_link_text() {
    let res = clean_link_text(STEAK_ENTRY);
    let exp_res = "
|6
|Steak
|Give one pet Steak.
Attack with +20 damage, once.
|Yes
|Yes
|No
";

    assert_eq!(res, exp_res)
}

#[test]
fn test_parse_food_entry() {
    let mut foods: Vec<FoodRecord> = vec![];
    let cols: Vec<FoodTableCols> = vec![
        FoodTableCols::Name,
        FoodTableCols::Tier,
        FoodTableCols::Effect,
        FoodTableCols::GamePack(Pack::Turtle),
        FoodTableCols::GamePack(Pack::Puppy),
        FoodTableCols::GamePack(Pack::Star),
    ];

    parse_one_food_entry(CUPCAKE_ENTRY, &cols, &mut foods).unwrap();
    assert_eq!(
        foods,
        vec![
            FoodRecord {
                name: FoodName::Cupcake,
                tier: 2,
                effect: "Give one pet +3 attack and +3 health until end of battle.".to_string(),
                pack: Pack::Turtle,
                holdable: false,
                single_use: true,
                end_of_battle: true,
                random: false,
                n_targets: 1,
                effect_atk: 3,
                effect_health: 3,
                turn_effect: false,
                cost: 3,
                img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/8/8a/Cupcake_Icon.png/revision/latest?cb=20230409195129")
            },
            FoodRecord {
                name: FoodName::Cupcake,
                tier: 2,
                effect: "Give one pet +3 attack and +3 health until end of battle.".to_string(),
                pack: Pack::Puppy,
                holdable: false,
                single_use: true,
                end_of_battle: true,
                random: false,
                n_targets: 1,
                effect_atk: 3,
                effect_health: 3,
                turn_effect: false,
                cost: 3,
                img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/8/8a/Cupcake_Icon.png/revision/latest?cb=20230409195129")
            }
        ]
    )
}

#[test]
fn test_parse_new_pack_or_extra_data_food_entry() {
    let mut foods: Vec<FoodRecord> = vec![];
    let cols: Vec<FoodTableCols> = vec![
        FoodTableCols::Name,
        FoodTableCols::Tier,
        FoodTableCols::Effect,
        FoodTableCols::GamePack(Pack::Turtle),
        FoodTableCols::GamePack(Pack::Puppy),
        FoodTableCols::GamePack(Pack::Star),
    ];

    // 6 columns expected but 7 items found.
    assert!(parse_one_food_entry(CUPCAKE_ENTRY_NEW_PACK, &cols, &mut foods).is_err());
}

#[test]
fn test_parse_missing_data_food_entry() {
    let mut foods: Vec<FoodRecord> = vec![];
    let cols: Vec<FoodTableCols> = vec![
        FoodTableCols::Name,
        FoodTableCols::Tier,
        FoodTableCols::Effect,
        FoodTableCols::GamePack(Pack::Turtle),
        FoodTableCols::GamePack(Pack::Puppy),
        FoodTableCols::GamePack(Pack::Star),
    ];

    // 6 columns expected but 5 items found.
    assert!(parse_one_food_entry(CUPCAKE_ENTRY_MISSING_FIELD, &cols, &mut foods).is_err());
}

#[test]
fn food_cost() {
    assert_eq!(3, get_food_cost("Cupcake"));
    assert_eq!(1, get_food_cost("Sleeping Pill"));
    assert_eq!(0, get_food_cost("Weak"))
}

#[test]
fn test_holdable_item() {
    assert!(is_holdable_item("Steak", &clean_link_text(STEAK_ENTRY)));
    assert!(!is_holdable_item(
        "Broccoli",
        &clean_link_text(BROCCOLI_ENTRY)
    ));
    // Exception. Only correct because matches name.
    assert!(is_holdable_item("Peanut", &clean_link_text(PEANUTS_ENTRY)));
    assert!(!is_holdable_item(
        "Not Peanuts",
        &clean_link_text(PEANUTS_ENTRY)
    ));
}

#[test]
fn test_single_use_item() {
    // Steak lasts multiple battles but only has one use during battle.
    assert_eq!(
        (false, true),
        is_temp_single_use("Steak", &clean_link_text(STEAK_ENTRY))
    );
    // Sushi is not temporary and has its effects persist during battle (technically)
    assert_eq!(
        (false, false),
        is_temp_single_use("Sushi", &clean_link_text(SUSHI_ENTRY))
    );
    // Cupcake is temporary and its effects end after battle so single use.
    assert_eq!(
        (true, true),
        is_temp_single_use("Cupcake", &clean_link_text(CUPCAKE_ENTRY))
    );
    // Exception. Matches only base on name.
    assert_eq!(
        (false, true),
        is_temp_single_use("Sleeping Pill", &clean_link_text(SLEEPING_PILL_ENTRY))
    );
    assert_eq!(
        (false, false),
        is_temp_single_use("Pill", &clean_link_text(SLEEPING_PILL_ENTRY))
    );
}

#[test]
fn test_random_and_n_effect() {
    // Sushi targets three friends and is random.
    assert_eq!(
        (true, 3),
        get_random_n_effect(&clean_link_text(SUSHI_ENTRY)).unwrap()
    );
    // Popcorns targets one friend and is random.
    assert_eq!(
        (true, 1),
        get_random_n_effect(&clean_link_text(POPCORNS_ENTRY)).unwrap()
    );
    // Sleeping pill targets one friend but is non-random.
    assert_eq!(
        (false, 1),
        get_random_n_effect(&clean_link_text(SLEEPING_PILL_ENTRY)).unwrap()
    )
}

#[test]
fn test_turn_based_effect() {
    // Sleeping pill is not turn-based.
    assert!(!is_turn_effect(&clean_link_text(SLEEPING_PILL_ENTRY)));
    // Carrot activates at the end of a turn.
    assert!(is_turn_effect(&clean_link_text(CARROT_ENTRY)));
    // While grape activates at the start of a turn.
    assert!(is_turn_effect(&clean_link_text(GRAPES_ENTRY)));
}

#[test]
fn test_get_atk_effect() {
    // Atk buffs or debuffs.
    assert_eq!(
        -1,
        get_effect_attack(&clean_link_text(BROCCOLI_ENTRY)).unwrap()
    );
    assert_eq!(
        20,
        get_effect_attack(&clean_link_text(STEAK_ENTRY)).unwrap()
    );
    // Dmg reduced.
    assert_eq!(
        2,
        get_effect_attack(&clean_link_text(GARLIC_ENTRY)).unwrap()
    );
    // Dmg dealt.
    assert_eq!(5, get_effect_attack(&clean_link_text(CHILI_ENTRY)).unwrap());
    // Summon attack.
    assert_eq!(1, get_effect_attack(&clean_link_text(HONEY_ENTRY)).unwrap());
    // Grapes gives gold not atk.
    assert_eq!(
        0,
        get_effect_attack(&clean_link_text(GRAPES_ENTRY)).unwrap()
    );
    // Sleeping pill doesn't have any attack related text.
    assert_eq!(
        0,
        get_effect_attack(&clean_link_text(SLEEPING_PILL_ENTRY)).unwrap()
    );
}

#[test]
fn test_get_health_effect() {
    // Health buffs or debuffs.
    assert_eq!(
        3,
        get_effect_health(&clean_link_text(BROCCOLI_ENTRY)).unwrap()
    );
    assert_eq!(
        -1,
        get_effect_health(&clean_link_text(SHRIMP_ENTRY)).unwrap()
    );
    // Summon health.
    assert_eq!(1, get_effect_health(&clean_link_text(HONEY_ENTRY)).unwrap());
    // Sleeping pill doesn't have any health related text.
    assert_eq!(
        0,
        get_effect_health(&clean_link_text(SLEEPING_PILL_ENTRY)).unwrap()
    );
}
