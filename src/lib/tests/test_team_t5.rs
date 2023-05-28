use itertools::Itertools;

use crate::{
    effects::{state::Position, stats::Statistics, trigger::TRIGGER_START_BATTLE},
    foods::names::FoodName,
    pets::names::PetName,
    shop::store::ShopItem,
    teams::{combat::TeamCombat, team::TeamFightOutcome, viewer::TeamViewer},
    tests::common::{
        count_pets, test_cricket_horse_team, test_crocodile_team, test_eagle_team, test_fox_team,
        test_goat_team, test_hamster_team, test_hyena_team, test_lion_highest_tier_team,
        test_lion_lowest_tier_team, test_lionfish_team, test_mammoth_team, test_microbe_team,
        test_monkey_team, test_moose_team, test_polar_bear_team, test_poodle_team, test_rhino_team,
        test_scorpion_team, test_seal_team, test_shark_team, test_shoebill_team,
        test_siberian_husky_team, test_skunk_team, test_swordfish_team, test_triceratops_team,
        test_turkey_team, test_vulture_team,
    },
    Entity, EntityName, Food, ItemCondition, Pet, Shop, ShopItemViewer, ShopViewer, Team,
    TeamEffects, TeamShopping,
};

#[test]
fn test_battle_croc_team() {
    let mut team = test_crocodile_team();
    let mut enemy_team = test_crocodile_team();

    let last_pet = team.last().unwrap();
    let last_enemy_pet = team.last().unwrap();
    assert_eq!(last_pet.read().unwrap().name, PetName::Cricket);
    assert_eq!(last_enemy_pet.read().unwrap().name, PetName::Cricket);

    // After start of battle, both crickets at end are sniped.
    // Two zombie crickets are spawned in their place.
    team.fight(&mut enemy_team).unwrap();

    let last_pet = team.nth(3).unwrap();
    let last_enemy_pet = enemy_team.nth(3).unwrap();

    assert_eq!(team.all().len(), 4);
    assert_eq!(enemy_team.all().len(), 4);
    assert_eq!(last_pet.read().unwrap().name, PetName::ZombieCricket);
    assert_eq!(last_enemy_pet.read().unwrap().name, PetName::ZombieCricket);
}

#[test]
fn test_battle_rhino_team() {
    let mut team = test_rhino_team();
    let mut enemy_team = test_cricket_horse_team();

    let outcome = team.fight(&mut enemy_team).unwrap();

    assert_eq!(outcome, TeamFightOutcome::None);
    // Only one damage from first cricket to trigger chain of faint triggers.
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 5,
            health: 7
        }
    );
    // All pets mowed down by rhino. After horse faints, zombie crickets spawn.
    assert!(enemy_team
        .all()
        .iter()
        .all(|pet| pet.read().unwrap().name == PetName::ZombieCricket))
}

#[test]
fn test_battle_rhino_against_token_team() {
    let bee = Pet::new(
        PetName::Bee,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();
    let duck = Pet::try_from(PetName::Duck).unwrap();

    let rhino = Pet::try_from(PetName::Rhino).unwrap();

    let mut t1 = Team::new(&[Some(duck), Some(bee)], 5).unwrap();
    let mut t2 = Team::new(&[Some(rhino)], 5).unwrap();

    t1.fight(&mut t2).unwrap();

    // Rhino does 8 damage to Bee because tier 1.
    assert_eq!(t1.first().unwrap().read().unwrap().tier, 1);
    assert_eq!(
        t1.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 50,
            health: 42
        }
    );
}

#[test]
fn test_battle_chili_rhino() {
    const RHINO_DMG: Statistics = Statistics {
        attack: 0,
        health: -5,
    };
    const RHINO_KNOCKOUT_DMG: Statistics = Statistics {
        attack: 0,
        health: -4,
    };

    let mut team = Team::new(&[Some(Pet::new(PetName::Rhino, None, 1).unwrap())], 5).unwrap();
    let chili = Food::try_from(FoodName::Chili).unwrap();
    team.set_item(&Position::First, Some(chili)).unwrap();

    let mut enemy_team = Team::new(
        &[
            Pet::new(PetName::Mammoth, None, 1).ok(),
            Pet::new(PetName::Dog, None, 1).ok(),
        ],
        5,
    )
    .unwrap();
    let mammoth = enemy_team.first().unwrap();
    let mammoth_start_stats = mammoth.read().unwrap().stats;

    team.fight(&mut enemy_team).unwrap();

    // Rhino knockout dmg triggers because dog dies behind mammoth.
    assert_eq!(
        mammoth_start_stats + RHINO_DMG + RHINO_KNOCKOUT_DMG,
        mammoth.read().unwrap().stats
    );
}

#[test]
fn test_shop_scorpion_team() {
    let mut team = Team::default();
    team.open_shop().unwrap();
    // Clear pets manually.
    team.shop.pets.clear();

    // No pets on team.
    assert!(team.first().is_none());
    // Add a scorpion to the shop manually.
    team.shop
        .add_item(ShopItem::new(Pet::try_from(PetName::Scorpion).unwrap()))
        .unwrap();

    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();
    // Scorpion summoned and gains peanuts after purchase.
    let scorpion = team.first().unwrap();
    assert_eq!(
        scorpion.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Peanut
    );
}

#[test]
fn test_battle_scorpion_team() {
    let mut team = test_scorpion_team();
    let mut enemy_team = test_skunk_team();

    // Replace peanut with mushroom.
    team.set_item(
        &Position::First,
        Some(Food::try_from(FoodName::Mushroom).unwrap()),
    )
    .unwrap();

    // Fight and kill mushroomed scorpion.
    team.fight(&mut enemy_team).unwrap();

    // Gets peanuts.
    let summoned_scorpion = team.first().unwrap();
    assert_eq!(
        summoned_scorpion
            .read()
            .unwrap()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Peanut
    );

    // Which enables a draw.
    let outcome = team.fight(&mut enemy_team).unwrap();
    assert_eq!(outcome, TeamFightOutcome::Draw);
}

#[test]
fn test_battle_shark_team() {
    let mut team = test_shark_team();
    let mut enemy_team = test_shark_team();

    let shark = team.last().unwrap();
    let shark_start_stats = shark.read().unwrap().stats;
    let n_team_crickets = count_pets(&team.friends, PetName::Cricket) as isize;

    // Lvl. 1 shark gains (1,2) on any faint.
    // 4 crickets so 8 total faint triggers.
    // 8 attack and 16 health gained.
    const SHARK_BUFF: Statistics = Statistics {
        attack: 1,
        health: 2,
    };

    for _ in 0..12 {
        team.fight(&mut enemy_team).unwrap();
    }

    let mut final_stats = SHARK_BUFF;
    final_stats.attack *= n_team_crickets * 2;
    final_stats.health *= n_team_crickets * 2;
    assert_eq!(shark_start_stats + final_stats, shark.read().unwrap().stats);
}

#[test]
fn test_battle_turkey_team() {
    let mut team = test_turkey_team();
    let mut enemy_team = test_turkey_team();

    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // Cricket faints, zombie version spawned, and it gains (3,3) (lvl.1 turkey)
    let zombie_cricket = team.first().unwrap();
    assert_eq!(
        zombie_cricket.read().unwrap().stats,
        Statistics {
            attack: 4,
            health: 4
        }
    );
}

#[test]
fn test_battle_hyena_team() {
    let hyena_pos = Position::First;
    let mut team = test_hyena_team();
    let mut enemy_team = test_cricket_horse_team();
    team.set_seed(Some(20));
    enemy_team.set_seed(Some(20));

    // Original positions
    assert_eq!(team.nth(1).unwrap().read().unwrap().name, PetName::Gorilla);
    assert_eq!(
        enemy_team.nth(2).unwrap().read().unwrap().name,
        PetName::Horse
    );

    let team_stats = team
        .friends
        .iter()
        .flatten()
        .map(|pet| pet.read().unwrap().stats)
        .collect_vec();
    let enemy_stats = enemy_team
        .friends
        .iter()
        .flatten()
        .map(|pet| pet.read().unwrap().stats)
        .collect_vec();

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // At lvl. 1 hyena swaps stats of all pets.
    for (mut og, new) in team
        .friends
        .iter()
        .flatten()
        .map(|pet| pet.read().unwrap().stats)
        .zip_eq(team_stats)
    {
        assert_eq!(og.invert().to_owned(), new)
    }
    for (mut og, new) in enemy_team
        .friends
        .iter()
        .flatten()
        .map(|pet| pet.read().unwrap().stats)
        .zip_eq(enemy_stats)
    {
        assert_eq!(og.invert().to_owned(), new)
    }

    // Reset teams.
    team.restore();
    enemy_team.restore();

    // Level up hyena.
    team.set_level(&hyena_pos, 2).unwrap();
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Hyena at lvl. 2 swaps positions of pets.
    assert_eq!(team.first().unwrap().read().unwrap().name, PetName::Gorilla);
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().name,
        PetName::Horse
    );

    // Reset teams.
    team.restore();
    enemy_team.restore();

    // Level up hyena.
    team.set_level(&hyena_pos, 3).unwrap();
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Hyena at lvl. 3 swaps positions and stats of pets.
    let gorilla = team.first().unwrap();
    let horse = enemy_team.first().unwrap();
    assert!(
        gorilla.read().unwrap().name == PetName::Gorilla
            && gorilla.read().unwrap().stats == Statistics::new(9, 6).unwrap()
    );
    assert!(
        horse.read().unwrap().name == PetName::Horse
            && horse.read().unwrap().stats == Statistics::new(1, 2).unwrap()
    );
}

#[test]
fn test_battle_lionfish_team() {
    let mut team = test_lionfish_team();
    let mut enemy_team = test_mammoth_team();

    // Enemy team's mammoth has no item.
    let mammoth = enemy_team.first().unwrap();
    assert_eq!(mammoth.read().unwrap().item, None);
    assert_eq!(
        mammoth.read().unwrap().stats,
        Statistics::new(3, 10).unwrap()
    );

    team.fight(&mut enemy_team).unwrap();

    // Prior to Dog at position 0 of team attacking, lionfish ability activates giving weakness to frontline mammoth.
    // Mammoth takes additional damage as a result.
    let mut example_weakness = Food::try_from(FoodName::Weak).unwrap();
    example_weakness.ability.assign_owner(Some(&mammoth));

    assert_eq!(mammoth.read().unwrap().item, Some(example_weakness));
    assert_eq!(
        mammoth.read().unwrap().stats,
        Statistics::new(3, 4).unwrap()
    );
}

#[test]
fn test_battle_eagle_team() {
    let mut team = test_eagle_team();
    let mut enemy_team = test_eagle_team();

    team.fight(&mut enemy_team).unwrap();

    let summoned_pet = team.first().unwrap();
    assert_eq!(summoned_pet.read().unwrap().tier, 6);
}

#[test]
fn test_battle_microbe_team() {
    let mut team = test_microbe_team();
    let mut enemy_team = test_eagle_team();

    team.fight(&mut enemy_team).unwrap();

    // All pets have weakness after microbe faints.
    for pet in team
        .friends
        .iter()
        .chain(enemy_team.friends.iter())
        .flatten()
    {
        assert_eq!(
            pet.read().unwrap().item.as_ref().unwrap().name,
            FoodName::Weak
        );
    }
}

#[test]
fn test_battle_lion_lowest_tier_team() {
    let mut team = test_lion_lowest_tier_team();
    let mut enemy_team = test_eagle_team();

    let highest_tier_pet = team
        .all()
        .into_iter()
        .max_by(|pet_1, pet_2| pet_1.read().unwrap().tier.cmp(&pet_2.read().unwrap().tier))
        .unwrap();
    // Highest tier pet not lion.
    assert_ne!(highest_tier_pet.read().unwrap().name, PetName::Lion);
    let lion = team.first().unwrap();
    let lion_original_stats = lion.read().unwrap().stats;

    // Activate start of battle effects.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Stats are unchanged.
    assert_eq!(
        lion_original_stats,
        team.first().unwrap().read().unwrap().stats
    )
}

#[test]
fn test_battle_lion_highest_tier_team() {
    let mut team = test_lion_highest_tier_team();
    let mut enemy_team = test_eagle_team();

    let highest_tier_pet = team
        .all()
        .into_iter()
        .max_by(|pet_1, pet_2| pet_1.read().unwrap().tier.cmp(&pet_2.read().unwrap().tier))
        .unwrap();
    // Highest tier pet is lion.
    assert_eq!(highest_tier_pet.read().unwrap().name, PetName::Lion);
    let lion = team.first().unwrap();
    let lion_original_stats = lion.read().unwrap().stats;

    // Activate start of battle effects.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Adds 50% of attack and health to original stats.
    assert_eq!(
        lion_original_stats + (lion_original_stats.mult_perc(&Statistics::new(50, 50).unwrap())),
        team.first().unwrap().read().unwrap().stats
    )
}

#[test]
fn test_battle_swordfish_team() {
    let mut team = test_swordfish_team();
    let mut enemy_team = test_eagle_team();

    let swordfish = team.first().unwrap();
    let eagle = enemy_team.first().unwrap();

    assert!(eagle.read().unwrap().stats.health == 5);
    assert!(swordfish.read().unwrap().stats.health == 25);

    // Activate start of battle effect.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Both swordfish and enemy eagle are hit and take 25 dmg.
    assert!(swordfish.read().unwrap().stats.health == 0);
    assert!(eagle.read().unwrap().stats.health == 0);
}

#[test]
fn test_battle_triceratops_team() {
    let mut team = test_triceratops_team();
    let mut enemy_team = test_cricket_horse_team();

    let triceratops = team.first().unwrap();
    let gorilla = team.nth(1).unwrap();

    assert_eq!(
        triceratops.read().unwrap().stats,
        Statistics::new(5, 6).unwrap()
    );
    assert_eq!(
        gorilla.read().unwrap().stats,
        Statistics::new(6, 9).unwrap()
    );

    team.fight(&mut enemy_team).unwrap();

    // Triceratops takes 1 dmg. Gorilla behind gets (3,3) buff.
    assert_eq!(
        triceratops.read().unwrap().stats,
        Statistics::new(5, 5).unwrap()
    );
    assert_eq!(
        gorilla.read().unwrap().stats,
        Statistics::new(9, 12).unwrap()
    );
}

#[test]
fn test_battle_vulture_team() {
    let mut team = test_vulture_team();
    let mut enemy_team = test_cricket_horse_team();
    enemy_team.set_seed(Some(25));

    // Three attack phases to reach two fainted pets.
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // Two fainted pets.
    assert_eq!(team.fainted.len(), 2);
    // Enemy team has an additional fainted pet because vulture effect triggers.
    assert_eq!(enemy_team.fainted.len(), 3);
}

#[test]
fn test_shop_cow_team() {
    let mut team = Team::default();

    // Create shop with Cow inside.
    let mut shop = Shop::default();
    shop.add_item(ShopItem::from(Pet::try_from(PetName::Cow).unwrap()))
        .unwrap();

    // Replace shop.
    team.replace_shop(shop).unwrap().open_shop().unwrap();

    // One item in shop
    assert_eq!(team.len_shop_foods(), 1);
    // Not milk.
    assert!(!team
        .shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Food)
        .unwrap()
        .iter()
        .any(|item| item.name() == EntityName::Food(FoodName::Milk)));

    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();
    // Two free milks added.
    assert_eq!(team.len_shop_foods(), 2);
    assert!(team
        .shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Food)
        .unwrap()
        .iter()
        .all(|item| item.name() == EntityName::Food(FoodName::Milk) && item.cost == 0))
}

#[test]
fn test_shop_monkey_team() {
    let mut team = test_monkey_team();

    team.open_shop().unwrap();

    let ant = team.first().unwrap();
    let ant_start_stats = ant.read().unwrap().stats;
    const MONKEY_BUFF: Statistics = Statistics {
        attack: 2,
        health: 3,
    };

    team.close_shop().unwrap();

    // Ant got monkey buff at end of turn.
    assert_eq!(ant.read().unwrap().stats, ant_start_stats + MONKEY_BUFF);
}

#[test]
fn test_shop_seal_team() {
    let mut team = test_seal_team();

    team.set_shop_seed(Some(12)).open_shop().unwrap();

    let pets = team.all();
    let [ant_1, ant_2, seal] = pets.get(0..3).unwrap() else { panic!() };
    let (ant_1_start_stats, ant_2_start_stats, seal_start_stats) = (
        ant_1.read().unwrap().stats,
        ant_2.read().unwrap().stats,
        seal.read().unwrap().stats,
    );
    const SEAL_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    team.buy(&Position::First, &Entity::Food, &Position::Last)
        .unwrap();

    // Two ants get buff after seal eats food.
    assert!(
        ant_1_start_stats + SEAL_BUFF == ant_1.read().unwrap().stats
            && ant_2_start_stats + SEAL_BUFF == ant_2.read().unwrap().stats
            && seal_start_stats == seal.read().unwrap().stats
    );
}

#[test]
fn test_shop_moose_team() {
    let mut team = test_moose_team();
    let seed = Some(12);
    team.set_seed(seed)
        .set_shop_tier(6)
        .unwrap()
        .set_shop_seed(seed)
        .open_shop()
        .unwrap();

    let (freeze_pos, freeze_item_type) = (Position::All(ItemCondition::None), Entity::Pet);
    team.freeze_shop(&freeze_pos, &freeze_item_type).unwrap();

    // Items are frozen before the end of the turn.
    let items = team
        .get_shop()
        .get_shop_items_by_pos(&freeze_pos, &freeze_item_type)
        .unwrap();
    assert!(items.iter().all(|item| item.is_frozen()));
    // Min pet tier in shop is 4.
    let min_tier = items
        .iter()
        .min_by(|item_1, item_2| item_1.tier().cmp(&item_2.tier()))
        .map(|item| item.tier())
        .unwrap();
    assert_eq!(min_tier, 4);

    const MOOSE_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    let mult_moose_buff = MOOSE_BUFF * Statistics::new(min_tier, min_tier).unwrap();
    let buff_target = team.nth(1).unwrap();
    let buff_target_start_stats = buff_target.read().unwrap().stats;

    // End turn.
    team.close_shop().unwrap();

    // Items no longer frozen.
    let items = team
        .get_shop()
        .get_shop_items_by_pos(&freeze_pos, &freeze_item_type)
        .unwrap();
    assert!(items.iter().all(|item| !item.is_frozen()));
    // Target gets (4,4) = (1,1) * 4.
    assert_eq!(
        buff_target.read().unwrap().stats,
        buff_target_start_stats + mult_moose_buff
    );
}

#[test]
fn test_shop_goat_team() {
    let mut team = test_goat_team();

    const PET_COST: usize = 3;
    const GOAT_REFUND: usize = 1;
    const START_GOLD: usize = 10;

    team.open_shop().unwrap();

    // Start with this much gold.
    assert_eq!(team.gold(), START_GOLD);

    let (buy_pos, item_type) = (Position::First, Entity::Pet);
    let found_items = team
        .get_shop()
        .get_shop_items_by_pos(&buy_pos, &item_type)
        .unwrap();
    // Pet costs 3 gold.
    assert_eq!(found_items.first().unwrap().cost, PET_COST);

    // Buy pet.
    team.buy(&buy_pos, &item_type, &Position::First).unwrap();

    assert_eq!(team.gold(), START_GOLD - PET_COST + GOAT_REFUND);
}

#[test]
fn test_shop_poodle_team() {
    let mut team = test_poodle_team();
    team.set_seed(Some(12)).open_shop().unwrap();

    let pets = team.all();
    let [
        ant,
        dog_1,
        dog_2,
        poodle,
        tiger
        ] = pets.get(0..5).unwrap() else { panic!() };

    const POODLE_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };

    let (
        ant_start_stats,
        dog_1_start_stats,
        dog_2_start_stats,
        poodle_start_stats,
        tiger_start_stats,
    ) = (
        ant.read().unwrap().stats,
        dog_1.read().unwrap().stats,
        dog_2.read().unwrap().stats,
        poodle.read().unwrap().stats,
        tiger.read().unwrap().stats,
    );
    team.close_shop().unwrap();

    // Only one dog buffed and poodle doesn't buff itself.
    assert!(
        ant.read().unwrap().stats == ant_start_stats + POODLE_BUFF
            && dog_1.read().unwrap().stats == dog_1_start_stats
            && dog_2.read().unwrap().stats == dog_2_start_stats + POODLE_BUFF
            && poodle.read().unwrap().stats == poodle_start_stats
            && tiger.read().unwrap().stats == tiger_start_stats + POODLE_BUFF
    );
}

#[test]
fn test_shop_fox_team() {
    let mut team = test_fox_team();
    let seed = Some(12);
    team.set_seed(seed).set_shop_seed(seed).open_shop().unwrap();

    let fox = team.first().unwrap();
    // Roll so no money left to buy from shop.
    for _ in 0..10 {
        team.roll_shop().unwrap();
    }
    // No gold.
    assert_eq!(team.gold(), 0);
    // Fox has no item.
    assert_eq!(fox.read().unwrap().item, None);

    team.close_shop().unwrap();

    // Stole honey from shop.
    assert_eq!(
        fox.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Honey
    );
}

#[test]
fn test_shop_hamster_team() {
    let mut team = test_hamster_team();

    team.open_shop().unwrap();

    const STARTING_GOLD: usize = 10;
    assert_eq!(team.gold(), STARTING_GOLD);

    // Each roll with lvl.1 hamster gives one gold back.
    // Works twice.
    for _ in 0..2 {
        team.roll_shop().unwrap();
        assert_eq!(team.gold(), STARTING_GOLD);
    }

    // Uses exhausted so now cost 1 gold.
    team.roll_shop().unwrap();
    assert_eq!(team.gold(), STARTING_GOLD - 1);
}

#[test]
fn test_shop_polar_bear_team() {
    let mut team = test_polar_bear_team();

    /// Helper function to get first pet stats from shop.
    fn first_shop_pet_stats(team: &Team, pos: &Position, item_type: &Entity) -> Statistics {
        let frozen_shop_pet = team
            .get_shop()
            .get_shop_items_by_pos(pos, item_type)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        Statistics::new(
            frozen_shop_pet.attack_stat().unwrap(),
            frozen_shop_pet.health_stat().unwrap(),
        )
        .unwrap()
    }

    let (shop_pet_pos, item_type) = (Position::First, Entity::Pet);
    team.open_shop().unwrap();

    let frozen_shop_pet_start_stats = first_shop_pet_stats(&team, &shop_pet_pos, &item_type);
    const POLAR_BEAR_BUFF: Statistics = Statistics {
        attack: 4,
        health: 4,
    };

    // Freeze pet.
    team.freeze_shop(&shop_pet_pos, &item_type)
        .unwrap()
        .close_shop()
        .unwrap();

    // First shop pet gets polar bear buff on start of turn.
    team.open_shop().unwrap();
    assert_eq!(
        frozen_shop_pet_start_stats + POLAR_BEAR_BUFF,
        first_shop_pet_stats(&team, &shop_pet_pos, &item_type)
    );
}

#[test]
fn test_shop_shoebill_team() {
    let mut team = test_shoebill_team();

    let affected_pos = Position::All(ItemCondition::None);

    team.open_shop().unwrap();

    // Close shop with no strawberries on friends.
    let friends_start_stats_no_berry = team
        .all()
        .into_iter()
        .map(|friend| friend.read().unwrap().stats)
        .collect_vec();

    team.close_shop().unwrap();

    // No change in stats on ending turn.
    assert_eq!(
        friends_start_stats_no_berry,
        team.all()
            .into_iter()
            .map(|friend| friend.read().unwrap().stats)
            .collect_vec()
    );

    team.open_shop().unwrap();

    // Give team strawberries.
    team.set_item(
        &affected_pos,
        Some(Food::try_from(FoodName::Strawberry).unwrap()),
    )
    .unwrap();

    let friends = team.all();
    let friends_start_stats_w_berry = friends
        .iter()
        .map(|friend| friend.read().unwrap().stats)
        .collect_vec();
    const SHOEBILL_BUFF: Statistics = Statistics {
        attack: 1,
        health: 2,
    };

    // All friends have strawberries.
    assert!(friends
        .iter()
        .all(|pet| pet.read().unwrap().item.as_ref().unwrap().name == FoodName::Strawberry));
    // End turn.
    team.close_shop().unwrap();

    let friends_curr_stats = friends
        .iter()
        .map(|friend| friend.read().unwrap().stats)
        .collect_vec();

    for (prev, after) in friends_start_stats_w_berry
        .iter()
        .zip_eq(friends_curr_stats.iter())
    {
        // Every pet (including shoebill) gets buff since have strawberry.
        assert_eq!(*prev + SHOEBILL_BUFF, *after)
    }
}

#[test]
fn test_shop_siberian_husky_team() {
    let mut team = test_siberian_husky_team();
    let pet_pos_w_no_item = Position::First;
    // Give ant at front garlic.
    team.open_shop()
        .unwrap()
        .set_item(
            &pet_pos_w_no_item,
            Some(Food::try_from(FoodName::Garlic).unwrap()),
        )
        .unwrap();

    let all_pets = team.all();
    let [ant, dog, husky, tiger] = all_pets.get(0..4).unwrap() else { panic!() };
    let (ant_start_stats, dog_start_stats, husky_start_stats, tiger_start_stats) = all_pets
        .iter()
        .map(|pet| pet.read().unwrap().stats)
        .collect_tuple()
        .unwrap();
    const HUSKY_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };

    team.close_shop().unwrap();

    // All pets w/o an item (excluding husky) get husky buff.
    assert!(
        ant.read().unwrap().stats == ant_start_stats
            && dog.read().unwrap().stats == dog_start_stats + HUSKY_BUFF
            && husky.read().unwrap().stats == husky_start_stats
            && tiger.read().unwrap().stats == tiger_start_stats + HUSKY_BUFF
    );
}

#[test]
fn test_shop_zebra_team() {
    let mut team = Team::new(&[Some(Pet::try_from(PetName::Ant).unwrap())], 5).unwrap();

    // Create shop with Cow inside.
    let mut shop = Shop::default();
    shop.add_item(ShopItem::from(Pet::try_from(PetName::Zebra).unwrap()))
        .unwrap();

    // Replace shop.
    team.replace_shop(shop).unwrap().open_shop().unwrap();

    let ant = team.first().unwrap();

    const ZEBRA_BUFF: Statistics = Statistics {
        attack: 2,
        health: 2,
    };
    let ant_start_stats = ant.read().unwrap().stats;

    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    // Ant gets buff when zebra bought.
    assert_eq!(ant.read().unwrap().stats, ant_start_stats + ZEBRA_BUFF);

    team.sell(&Position::First).unwrap();

    // Ant gets additional buff when zebra sold.
    assert_eq!(
        ant.read().unwrap().stats,
        ant_start_stats + ZEBRA_BUFF + ZEBRA_BUFF
    );
}
