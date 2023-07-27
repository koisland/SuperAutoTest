use crate::{
    db::record::ToyRecord, toys::names::ToyName,
    wiki_scraper::parse_hard_mode_toys::parse_hard_mode_toys_table,
};

const HARD_MODE_TOYS_TABLE: &str = r#"
{| class="sortable wikitable"
! class="sortable" style="text-align: center;" | Name
! class="sortable" style="text-align: center;" | Trigger
! class="unsortable" colspan="2" | Effect
|-
<!-- TIER 2 -->
| {{IconSAP|Action Figure|size=40px}}
| Start of battle
| Give {{IconSAP|Coconut}} to enemies front-to-back for every second shop tier.
|-

| {{IconSAP|Dice|size=40px}}
| Roll
| Lose 1 {{IconSAP|Gold}}
|}
    "#;

#[test]
fn test_parse_hard_mode_toys() {
    let mut toys = Vec::new();
    let exp_toys = [
        ToyRecord {
            name: ToyName::ActionFigure,
            tier: 2,
            effect_trigger: Some("Start of battle".to_owned()),
            effect: Some(
                "Give Coconut to enemies front-to-back for every second shop tier.".to_owned(),
            ),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 1,
            temp_effect: false,
            lvl: 1,
            source: None,
            img_url: "https://superautopets.wiki.gg/images/e/ed/Action_Figure_Icon.png".to_owned(),
            hard_mode: true,
        },
        ToyRecord {
            name: ToyName::Dice,
            tier: 2,
            effect_trigger: Some("Roll".to_owned()),
            effect: Some("Lose 1 Gold".to_owned()),
            effect_atk: 0,
            effect_health: 0,
            n_triggers: 1,
            temp_effect: false,
            lvl: 1,
            source: None,
            img_url: "https://superautopets.wiki.gg/images/a/ad/Dice_Icon.png".to_owned(),
            hard_mode: true,
        },
    ];
    parse_hard_mode_toys_table(HARD_MODE_TOYS_TABLE, 2, &mut toys);
    assert_eq!(toys, exp_toys)
}
