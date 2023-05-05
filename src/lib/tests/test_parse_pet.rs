use crate::db::{pack::Pack, record::PetRecord};
use crate::wiki_scraper::{
    common::remove_icon_names,
    parse_pet::{
        parse_pet_effect_trigger, parse_pet_effects, parse_pet_packs, parse_pet_stats,
        parse_single_pet,
    },
};
use crate::PetName;

const MAMMOTH_ENTRY: &str = "
{{:Pets/row
    | pet = {{IconSAP|Mammoth|size=40px}}
    | attack = 3 | health = 10
    | turtlepack = yes | puppypack = yes
    | '''Faint''' → Give all friends +2 {{IconSAP|attack|nolink=yes}} and +2 {{IconSAP|health|nolink=yes}}.
    | '''Faint''' → Give all friends +4 {{IconSAP|attack|nolink=yes}} and +4 {{IconSAP|health|nolink=yes}}.
    | '''Faint''' → Give all friends +6 {{IconSAP|attack|nolink=yes}} and +6 {{IconSAP|health|nolink=yes}}.
    }}
";

const SLOTH_ENTRY: &str = "
{{:Pets/row
    | pet = {{IconSAP|Sloth|size=40px}}
    | attack = 1 | health = 1
    | turtlepack = yes | puppypack = yes | starpack = yes
    | ''\"Sloth has no special ability. Is kind of lame combat-wise. But he truly believes in you!\"''
    | ''\"Sloth has no special ability. Is kind of lame combat-wise. But he truly believes in you!\"''
    | ''\"Sloth has no special ability. Is kind of lame combat-wise. But he truly believes in you!\"''
    }}
    |}
";

const TIGER_ENTRY: &str = "
{{:Pets/row
    | pet = {{IconSAP|Tiger|size=40px}}
    | attack = 4 | health = 3
    | turtlepack = yes | puppypack = yes
    | The friend ahead repeats their ability in battle as if they were level 1.
    | The friend ahead repeats their ability in battle as if they were level 2.
    | The friend ahead repeats their ability in battle as if they were level 3.
    }}
";

#[test]
fn test_parse_stats() {
    let stats = parse_pet_stats(MAMMOTH_ENTRY).unwrap();
    assert_eq!(stats, (3, 10))
}

#[test]
fn test_parse_packs() {
    let mammoth_packs = parse_pet_packs(MAMMOTH_ENTRY);
    let sloth_packs = parse_pet_packs(SLOTH_ENTRY);
    assert_eq!(mammoth_packs, vec![Pack::Turtle, Pack::Puppy]);
    assert_eq!(sloth_packs, vec![Pack::Turtle, Pack::Puppy, Pack::Star]);
}

#[test]
fn test_parse_effect_triggers() {
    let effect_trigger_mammoth = parse_pet_effect_trigger(MAMMOTH_ENTRY);
    let effect_trigger_sloth = parse_pet_effect_trigger(SLOTH_ENTRY);
    let effect_trigger_tiger = parse_pet_effect_trigger(TIGER_ENTRY);

    assert_eq!(effect_trigger_mammoth, Some("Faint".to_string()));
    assert_eq!(effect_trigger_sloth, None);
    assert_eq!(effect_trigger_tiger, None);
}

#[test]
fn test_parse_effects() {
    let effect_trigger_mammoth = parse_pet_effect_trigger(MAMMOTH_ENTRY);
    let effect_trigger_sloth = parse_pet_effect_trigger(SLOTH_ENTRY);
    let effect_trigger_tiger = parse_pet_effect_trigger(TIGER_ENTRY);

    let effect_mammoth = parse_pet_effects(
        &remove_icon_names(MAMMOTH_ENTRY),
        effect_trigger_mammoth.is_some(),
    );
    let effect_sloth = parse_pet_effects(
        &remove_icon_names(SLOTH_ENTRY),
        effect_trigger_sloth.is_some(),
    );
    let effect_tiger = parse_pet_effects(
        &remove_icon_names(TIGER_ENTRY),
        effect_trigger_tiger.is_some(),
    );

    assert_eq!(
        effect_mammoth,
        vec![
            "Give all friends +2 attack and +2 health.",
            "Give all friends +4 attack and +4 health.",
            "Give all friends +6 attack and +6 health."
        ]
    );
    assert_eq!(
        effect_sloth,
        vec!["Sloth has no special ability. Is kind of lame combat-wise. But he truly believes in you!"; 3]
    );

    assert_eq!(
        effect_tiger,
        vec![
            "The friend ahead repeats their ability in battle as if they were level 1.",
            "The friend ahead repeats their ability in battle as if they were level 2.",
            "The friend ahead repeats their ability in battle as if they were level 3."
        ]
    );
}

#[test]
fn test_create_pet_record() {
    let mut curr_tier: usize = 6;
    let mut pets: Vec<PetRecord> = vec![];
    parse_single_pet(MAMMOTH_ENTRY, &mut curr_tier, &mut pets).unwrap();

    let exp_pets = vec![
        PetRecord {
            name: PetName::Mammoth,
            tier: 6,
            attack: 3,
            health: 10,
            pack: Pack::Turtle,
            effect_trigger: Some("Faint".to_string()),
            effect: Some("Give all friends +2 attack and +2 health.".to_string()),
            effect_atk: 2,
            effect_health: 2,
            n_triggers: 1,
            temp_effect: false,
            lvl: 1,
            cost: 3,
            img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/f/fe/Mammoth_Icon.png/revision/latest?cb=20230307093550"),
        },
        PetRecord {
            name: PetName::Mammoth,
            tier: 6,
            attack: 3,
            health: 10,
            pack: Pack::Turtle,
            effect_trigger: Some("Faint".to_string()),
            effect: Some("Give all friends +4 attack and +4 health.".to_string()),
            effect_atk: 4,
            effect_health: 4,
            n_triggers: 1,
            temp_effect: false,
            lvl: 2,
            cost: 3,
            img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/f/fe/Mammoth_Icon.png/revision/latest?cb=20230307093550"),
        },
        PetRecord {
            name: PetName::Mammoth,
            tier: 6,
            attack: 3,
            health: 10,
            pack: Pack::Turtle,
            effect_trigger: Some("Faint".to_string()),
            effect: Some("Give all friends +6 attack and +6 health.".to_string()),
            effect_atk: 6,
            effect_health: 6,
            n_triggers: 1,
            temp_effect: false,
            lvl: 3,
            cost: 3,
            img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/f/fe/Mammoth_Icon.png/revision/latest?cb=20230307093550"),
        },
        PetRecord {
            name: PetName::Mammoth,
            tier: 6,
            attack: 3,
            health: 10,
            pack: Pack::Puppy,
            effect_trigger: Some("Faint".to_string()),
            effect: Some("Give all friends +2 attack and +2 health.".to_string()),
            effect_atk: 2,
            effect_health: 2,
            n_triggers: 1,
            temp_effect: false,
            lvl: 1,
            cost: 3,
            img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/f/fe/Mammoth_Icon.png/revision/latest?cb=20230307093550"),
        },
        PetRecord {
            name: PetName::Mammoth,
            tier: 6,
            attack: 3,
            health: 10,
            pack: Pack::Puppy,
            effect_trigger: Some("Faint".to_string()),
            effect: Some("Give all friends +4 attack and +4 health.".to_string()),
            effect_atk: 4,
            effect_health: 4,
            n_triggers: 1,
            temp_effect: false,
            lvl: 2,
            cost: 3,
            img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/f/fe/Mammoth_Icon.png/revision/latest?cb=20230307093550"),
        },
        PetRecord {
            name: PetName::Mammoth,
            tier: 6,
            attack: 3,
            health: 10,
            pack: Pack::Puppy,
            effect_trigger: Some("Faint".to_string()),
            effect: Some("Give all friends +6 attack and +6 health.".to_string()),
            effect_atk: 6,
            effect_health: 6,
            n_triggers: 1,
            temp_effect: false,
            lvl: 3,
            cost: 3,
            img_url: String::from("https://static.wikia.nocookie.net/superautopets/images/f/fe/Mammoth_Icon.png/revision/latest?cb=20230307093550"),
        },
    ];
    assert_eq!(
        6,
        pets.iter()
            .zip(exp_pets.iter())
            .filter(|&(a, b)| a == b)
            .count()
    )
}
