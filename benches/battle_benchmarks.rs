use std::time::Duration;

use saptest::{teams::team::TeamFightOutcome, Pet, PetName, Team, TeamCombat};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

fn bench_blowfish_rally(c: &mut Criterion) {
    let team = |name: &str| {
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
        team.set_name(name).unwrap();
        team
    };
    let mut grp = c.benchmark_group("blowfish");
    grp.bench_function("blowfish_rally", |b| {
        b.iter_batched(
            || (team("1"), team("2")),
            |(mut team, mut enemy_team)| {
                let mut outcome = team.fight(&mut enemy_team).unwrap();
                while let TeamFightOutcome::None = outcome {
                    outcome = team.fight(&mut enemy_team).unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
    grp.finish();
}

/// Benchmark from https://github.com/manny405/sapai#battles.
fn bench_sapai(c: &mut Criterion) {
    let team = |name: &str| {
        let pets = [
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ox).unwrap()),
            Some(Pet::try_from(PetName::Tiger).unwrap()),
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(Some(50));
        team.set_name(name).unwrap();
        team
    };
    let enemy_team = |name: &str| {
        let pets = [
            Some(Pet::try_from(PetName::Sheep).unwrap()),
            Some(Pet::try_from(PetName::Tiger).unwrap()),
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(Some(50));
        team.set_name(name).unwrap();
        team
    };

    let mut grp = c.benchmark_group("sapai");
    grp.bench_function("sapai_example", |b| {
        b.iter_batched(
            || (team("1"), enemy_team("2")),
            |(mut team, mut enemy_team)| {
                let mut outcome = team.fight(&mut enemy_team).unwrap();
                while let TeamFightOutcome::None = outcome {
                    outcome = team.fight(&mut enemy_team).unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
    grp.finish()
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(60));
    targets = bench_blowfish_rally, bench_sapai
);
criterion_main!(benches);
