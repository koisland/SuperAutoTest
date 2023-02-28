use saptest::{teams::team::TeamFightOutcome, Pet, PetName, Team, TeamCombat};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_blowfish_rally(c: &mut Criterion) {
    let mut team = black_box({
        let mut blowfish = Pet::try_from(PetName::Blowfish).unwrap();
        blowfish.stats.health = 50;
        let hedgehog = Pet::try_from(PetName::Hedgehog).unwrap();
        let pets = [
            Some(hedgehog),
            Some(blowfish.clone()),
            Some(blowfish.clone()),
            Some(blowfish.clone()),
            Some(blowfish),
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(Some(50));
        team
    });
    let mut enemy_team = black_box(team.clone());

    c.bench_function("blowfish_rally", |b| {
        b.iter(|| {
            let mut outcome = team.fight(&mut enemy_team).unwrap();
            while let TeamFightOutcome::None = outcome {
                outcome = team.fight(&mut enemy_team).unwrap();
            }
        })
    });
}

/// Benchmark from https://github.com/manny405/sapai#battles.
fn bench_sapai(c: &mut Criterion) {
    let mut team = black_box({
        let pets = [
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ox).unwrap()),
            Some(Pet::try_from(PetName::Tiger).unwrap()),
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(Some(50));
        team
    });
    let mut enemy_team = black_box({
        let pets = [
            Some(Pet::try_from(PetName::Sheep).unwrap()),
            Some(Pet::try_from(PetName::Tiger).unwrap()),
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(Some(50));
        team
    });
    c.bench_function("sapai_example", |b| {
        b.iter(|| {
            let mut outcome = team.fight(&mut enemy_team).unwrap();
            while let TeamFightOutcome::None = outcome {
                outcome = team.fight(&mut enemy_team).unwrap();
            }
        })
    });
}

criterion_group!(benches, bench_blowfish_rally, bench_sapai);
criterion_main!(benches);
