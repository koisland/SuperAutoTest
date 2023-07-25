use crate::{
    db::{pack::Pack, record::FoodRecord},
    wiki_scraper::{common::clean_link_text, parse_ailment::parse_one_ailment_entry},
    FoodName,
};

const INK_ENTRY: &str = r#"
|{{IconSAP|Ink}}
|Attack and ability deal 3 less damage.
|Inflicted by {{IconSAP|Squid}} and {{IconSAP|Cuttlefish}} upon faint.
Inflicted on friendly pets by the {{IconSAP|Pen}} Toy in Hard Mode.
"#;

#[test]
fn test_parse_ailment() {
    let mut ailments = vec![];
    let entry = clean_link_text(INK_ENTRY);
    parse_one_ailment_entry(&entry, &mut ailments).unwrap();
    assert_eq!(
        ailments,
        vec![FoodRecord {
            name: FoodName::Ink,
            tier: 0,
            effect: String::from("Attack and ability deal 3 less damage."),
            pack: Pack::Unknown,
            holdable: true,
            single_use: false,
            end_of_battle: true,
            random: false,
            n_targets: 1,
            effect_atk: 3,
            effect_health: 0,
            turn_effect: false,
            cost: 0,
            img_url: String::from("https://superautopets.wiki.gg/images/a/a9/Ink_Icon.png"),
            is_ailment: true
        }]
    )
}
