use itertools::Itertools;

use crate::{
    common::pack::Pack,
    wiki_scraper::parse_food::{clean_link_text, get_cols, get_largest_table, FoodTableCols},
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

const FOOD_ENTRY: &str = "
|6
|{{IconSAP|Steak}}
|Give one [[Pets|pet]] [[Steak]].
Attack with +20 damage, once.
|Yes
|Yes
|No
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

    let cols = get_cols(table.first().unwrap()).unwrap();
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
    let res = clean_link_text(FOOD_ENTRY);
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
