use crate::{teams::team::TeamFightOutcome, Pet, PetName, Team, TeamCombat, TeamViewer};

#[test]
fn test_golden_effect() {
    let pets = vec![
        Some(Pet::try_from(PetName::Ant).unwrap()),
        Some(Pet::try_from(PetName::Ant).unwrap()),
    ];
    let enemy_pets = pets.clone();

    let mut team = Team::new(&pets, 5).unwrap();
    let mut enemy_team = Team::new(&enemy_pets, 5).unwrap();

    team.counters
        .entry("Trumpets".to_owned())
        .and_modify(|trumpets| *trumpets += 5);

    let mut outcome = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team).unwrap()
    }

    // Friend team wins because of added trumpets spawning a golden retriever.
    assert_eq!(outcome, TeamFightOutcome::Win);
    assert_eq!(
        team.first().unwrap().read().unwrap().name,
        PetName::GoldenRetriever
    );
}
