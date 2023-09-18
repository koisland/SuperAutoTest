use crate::{
    effects::state::EqualityCondition,
    teams::team::TeamFightOutcome,
    tests::common::{test_ant_team, test_scorpion_team},
    Entity, EntityName, FoodName, ItemCondition, PetName, Position, TeamCombat, TeamEffects,
    TeamShopping, TeamViewer, Toy, ToyName,
};

#[test]
fn test_toy_action_figure() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    const TIER: usize = 4;
    let exp_coconuts = TIER / 2;
    team.set_shop_tier(TIER).unwrap();
    team.toys
        .push(Toy::try_from(ToyName::ActionFigure).unwrap());

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // Two enemies get coconut from shop tier / 2.
    let enemy_pets_w_coconut = enemy_team.get_pets_by_cond(&ItemCondition::Equal(
        EqualityCondition::Name(EntityName::Food(FoodName::Coconut)),
    ));
    assert_eq!(enemy_pets_w_coconut.len(), exp_coconuts);
}

#[test]
fn test_toy_dice() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::Dice).unwrap());
    team.open_shop().unwrap();

    // Starting gold.
    assert_eq!(10, team.gold());

    team.roll_shop().unwrap();

    // Lose two gold per roll instead of one.
    assert_eq!(8, team.gold());
}

#[test]
fn test_toy_open_piggy_bank() {
    let mut team = test_ant_team();

    team.toys
        .push(Toy::try_from(ToyName::OpenPiggyBank).unwrap());

    team.open_shop().unwrap();

    // Starting gold.
    assert_eq!(10, team.gold());
    // Buy a 3-gold pet.
    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();
    // Lose one additional gold per purchase.
    assert_eq!(6, team.gold())
}

#[test]
fn test_toy_rubber_duck() {
    let mut team = test_scorpion_team();
    let mut enemy_team = test_scorpion_team();
    team.toys.push(Toy::try_from(ToyName::RubberDuck).unwrap());

    let outcome = team.fight(&mut enemy_team).unwrap();

    // Lose fight because of rubber duck.
    assert_eq!(outcome, TeamFightOutcome::Loss);
    let enemy_first_pet = enemy_team.first().unwrap();
    assert_eq!(enemy_first_pet.read().unwrap().name, PetName::Duck);
}
