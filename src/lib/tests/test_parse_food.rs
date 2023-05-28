use crate::{
    db::{pack::Pack, record::FoodRecord},
    regex_patterns::RGX_FOOD_ROW,
    wiki_scraper::{
        common::get_largest_table,
        parse_food::{
            clean_link_text, get_effect_attack, get_effect_health, get_food_cost,
            get_random_n_effect, is_holdable_item, is_temp_single_use, is_turn_effect,
            parse_one_food_entry,
        },
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
{{:Foods/row
| food = {{IconSAP|Steak}}
| turtlepack = yes | puppypack = yes
| ability = Give one pet {{IconSAP|Steak}}. Attack with +20 damage, once.
}}
";

const CUPCAKE_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Cupcake}}
| turtlepack = yes | puppypack = yes
| ability = Give one pet +3 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}} until end of battle.
}}
";

const BROCCOLI_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Broccoli}}
| starpack = yes
| ability = Give one pet -1 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}}.
}}
";

const SHRIMP_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Fried Shrimp}}
| starpack = yes
| ability = Give one pet +3 {{IconSAP|attack|nolink=yes}} and -1 {{IconSAP|health|nolink=yes}}.
}}
";

const GARLIC_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Garlic}}
| turtlepack = yes | puppypack = yes
| ability = Give one pet {{IconSAP|Garlic}}. Take 2 less damage.
}}
";

const GRAPES_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Grapes}}
| starpack = yes
| ability = Give one pet {{IconSAP|Grapes}}. Gain +1 {{IconSAP|Gold}} at the start of every turn.
}}
";

const CARROT_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Carrot}}
| starpack = yes
| ability = Give one pet {{IconSAP|Carrot}}. Gain +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}} at end of turn.
}}
";

const SUSHI_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Sushi}}
| turtlepack = yes | puppypack = yes
| ability = Give three random pets +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}}.
}}
";

const PEANUTS_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Peanut}}
| turtlepack = summoned | puppypack = summoned | starpack = summoned
| ability = Knockout any pet attacked and hurt by this.
}}
";

const SLEEPING_PILL_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Sleeping Pill}}
| turtlepack = yes | puppypack = yes
| ability = Make one pet faint. Always on sale!
}}
";

const CHILI_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Chili}}
| turtlepack = yes | puppypack = yes
| ability = Give one {{IconSAP|Chili}}. Attack second enemy for 5 {{IconSAP|damage|nolink=yes}}.
}}
";

const HONEY_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Honey}}
| turtlepack = yes | puppypack = yes
| ability = Give one pet {{IconSAP|Honey}}. Summon a 1/1 {{IconSAP|Bee}} after fainting.
}}
";

const POPCORNS_ENTRY: &str = "
{{:Foods/row
| food = {{IconSAP|Popcorns}}
| starpack = yes
| ability = Give one pet {{IconSAP|Popcorns}}. Summon a random pet from the same tier after fainting.
}}
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
fn test_clean_link_text() {
    let res = clean_link_text(STEAK_ENTRY);
    let exp_res = "
{{:Foods/row
| food = Steak
| turtlepack = yes | puppypack = yes
| ability = Give one pet Steak. Attack with +20 damage, once.

";

    assert_eq!(res, exp_res)
}

#[test]
fn test_parse_food_entry() {
    let mut foods: Vec<FoodRecord> = vec![];

    let cupcake_row = RGX_FOOD_ROW
        .find(CUPCAKE_ENTRY)
        .map(|mtch| mtch.as_str())
        .unwrap();
    parse_one_food_entry(cupcake_row, 2, &mut foods).unwrap();
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
                img_url: String::from(
                    "https://static.wikia.nocookie.net/superautopets/images/8/8a/Cupcake_Icon.png"
                )
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
                img_url: String::from(
                    "https://static.wikia.nocookie.net/superautopets/images/8/8a/Cupcake_Icon.png"
                )
            }
        ]
    )
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
