use itertools::Itertools;

use crate::{
    db::record::ToyRecord, regex_patterns::RGX_TIER_TABLE, toys::names::ToyName,
    wiki_scraper::parse_toy::parse_toys_table,
};

const TOY_TABLE: &str = r#"
{| class="sortable wikitable" style="width: 100%; margin-bottom: 1em;"
|-
! rowspan="2" width="17.5%" | Name
! colspan="2" | Ability
! rowspan="2" width="10%" | Source
|-
! width="5%" | Level
! Description
|-

<!-- TIER 1 -->
{{:Toys/row
| name = {{IconSAP|Balloon|size=40px}}
| source = {{IconSAP|Ferret}}
| '''On break''' → Give +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}} to the right-most friend.
| '''On break''' → Give +2 {{IconSAP|attack|nolink=yes}} and +2 {{IconSAP|health|nolink=yes}} to the right-most friend.
| '''On break''' → Give +3 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}} to the right-most friend.
}}

{{:False/row
| name = {{IconSAP|Balloon|size=40px}}
| source = {{IconSAP|Ferret}}
| '''On break''' → Give +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}} to the right-most friend.
| '''On break''' → Give +2 {{IconSAP|attack|nolink=yes}} and +2 {{IconSAP|health|nolink=yes}} to the right-most friend.
| '''On break''' → Give +3 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}} to the right-most friend.
}}

|}
"#;

const TOY_TABS: &str = r#"
<tabber>
|-|Tier 1 =

=== {{IconSAP|Tier 1|name=Tier 1 Toys|nolink=yes}} ===


{| class="sortable wikitable" style="width: 100%; margin-bottom: 1em;"
|-
! rowspan="2" width="17.5%" | Name
! colspan="2" | Ability
! rowspan="2" width="10%" | Source
|-
! width="5%" | Level
! Description
|-

<!-- TIER 1 -->
{{:Toys/row
| name = {{IconSAP|Balloon|size=40px}}
| source = {{IconSAP|Ferret}}
| '''On break''' → Give +1 {{IconSAP|attack|nolink=yes}} and +1 {{IconSAP|health|nolink=yes}} to the right-most friend.
| '''On break''' → Give +2 {{IconSAP|attack|nolink=yes}} and +2 {{IconSAP|health|nolink=yes}} to the right-most friend.
| '''On break''' → Give +3 {{IconSAP|attack|nolink=yes}} and +3 {{IconSAP|health|nolink=yes}} to the right-most friend.
}}

|}

|-|Tier 2 =

=== {{IconSAP|Tier 2|name=Tier 2 Toys|nolink=yes}} ===

{| class="sortable wikitable" style="width: 100%; margin-bottom: 1em;"
|-
! rowspan="2" width="17.5%" | Name
! colspan="2" | Ability
! rowspan="2" width="10%" | Source
|-
! width="5%" | Level
! Description
|-

<!-- TIER 2 -->
{{:Toys/row
| name = {{IconSAP|Radio}}
| source = {{IconSAP|Lemur}}
| '''On break''' → Give +1 {{IconSAP|health|nolink=yes}} to all friends.
| '''On break''' → Give +2 {{IconSAP|health|nolink=yes}} to all friends.
| '''On break''' → Give +3 {{IconSAP|health|nolink=yes}} to all friends.
}}

|}
</tabber>
"#;

#[test]
fn test_parse_toys() {
    let mut toys: Vec<ToyRecord> = vec![];
    let mut exp_toys = (1..=3)
        .map(|lvl| ToyRecord {
            name: ToyName::Balloon,
            tier: 1,
            effect_trigger: Some("On break".to_owned()),
            effect: Some(format!(
                "Give +{lvl} attack and +{lvl} health to the right-most friend."
            )),
            effect_atk: lvl,
            effect_health: lvl,
            n_triggers: 1,
            temp_effect: false,
            lvl,
            source: Some("Ferret".to_owned()),
            img_url: "https://superautopets.wiki.gg/images/b/bf/Balloon_Icon.png".to_owned(),
            hard_mode: false,
        })
        .collect_vec();
    exp_toys.extend((1..=3).map(|lvl| ToyRecord {
        name: ToyName::Radio,
        tier: 2,
        effect_trigger: Some("On break".to_owned()),
        effect: Some(format!("Give +{lvl} health to all friends.")),
        effect_atk: 0,
        effect_health: lvl,
        n_triggers: 1,
        temp_effect: false,
        lvl,
        source: Some("Lemur".to_owned()),
        img_url: "https://superautopets.wiki.gg/images/5/51/Radio_Icon.png".to_owned(),
        hard_mode: false,
    }));

    for cap in RGX_TIER_TABLE.captures_iter(TOY_TABS) {
        if let (Some(tier), Some(table)) = (
            cap.get(1)
                .and_then(|mtch| mtch.as_str().parse::<usize>().ok()),
            cap.get(2).map(|mtch| mtch.as_str()),
        ) {
            parse_toys_table(table, tier, &mut toys)
        }
    }

    assert_eq!(toys, exp_toys)
}

#[test]
fn test_parse_toy_table_rows() {
    let mut toys = Vec::new();
    let exp_toys = (1..=3)
        .map(|lvl| ToyRecord {
            name: ToyName::Balloon,
            tier: 1,
            effect_trigger: Some("On break".to_owned()),
            effect: Some(format!(
                "Give +{lvl} attack and +{lvl} health to the right-most friend."
            )),
            effect_atk: lvl,
            effect_health: lvl,
            n_triggers: 1,
            temp_effect: false,
            lvl,
            source: Some("Ferret".to_owned()),
            img_url: "https://superautopets.wiki.gg/images/b/bf/Balloon_Icon.png".to_owned(),
            hard_mode: false,
        })
        .collect_vec();

    parse_toys_table(TOY_TABLE, 1, &mut toys);

    assert_eq!(toys, exp_toys)
}
