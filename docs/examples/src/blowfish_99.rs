use saptest::{teams::team::TeamFightOutcome, Pet, PetName, Team, TeamCombat};


pub fn ninety_nine_blowfish_battle() -> Team {
    // 99 blowfish battle and a hedgehog at the front.
    let mut blowfish = Pet::try_from(PetName::Blowfish).unwrap();
    blowfish.stats.health = 50;
    let hedgehog = Pet::try_from(PetName::Hedgehog).unwrap();
    let mut pets = vec![Some(blowfish); 5];
    pets.insert(0, Some(hedgehog));

    let mut team = Team::new(&pets, 100).unwrap();
    let mut enemy_team = team.clone();

    let mut outcome = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team).unwrap();
    }
    team
}
