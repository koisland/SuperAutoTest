use crate::{
    db::record::ToyRecord,
    regex_patterns::{RGX_TIER_TABLE, RGX_TOY_NAME, RGX_TOY_ROW, RGX_TOY_SOURCE},
    wiki_scraper::parse_toy::parse_toys_table,
};

const TOY_TABS: &str = r#"
<tabber>
|-|Tier 1 =

=== {{IconSAP|Tier 1|name=Tier 1 Toys|nolink=yes}} ===


{| class="sortable fandom-table" style="width: 100%; margin-bottom: 1em;"
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

{| class="sortable fandom-table" style="width: 100%; margin-bottom: 1em;"
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

    for cap in RGX_TIER_TABLE.captures_iter(TOY_TABS) {
        if let (Some(tier), Some(table)) = (
            cap.get(1)
                .and_then(|mtch| mtch.as_str().parse::<usize>().ok()),
            cap.get(2).map(|mtch| mtch.as_str()),
        ) {
            parse_toys_table(table, tier, &mut toys)
        }
    }

    println!("{:?}", toys)
}
