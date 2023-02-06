use sapt::{battle::state::TeamFightOutcome, Pet, PetName, Team};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_blowfish_rally(c: &mut Criterion) {
    let mut team = black_box({
        let mut blowfish = Pet::try_from(PetName::Blowfish).unwrap();
        blowfish.stats.health = 50;
        let hedgehog = Pet::try_from(PetName::Hedgehog).unwrap();
        let pets = [
            hedgehog,
            blowfish.clone(),
            blowfish.clone(),
            blowfish.clone(),
            blowfish,
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(50);
        team
    });
    let mut enemy_team = black_box(team.clone());

    c.bench_function("blowfish_rally", |b| {
        b.iter(|| {
            let mut outcome = team.fight(&mut enemy_team);
            while let TeamFightOutcome::None = outcome {
                outcome = team.fight(&mut enemy_team);
            }
        })
    });
}

/// Benchmark from https://github.com/manny405/sapai#battles.
fn bench_sapai(c: &mut Criterion) {
    let mut team = black_box({
        let pets = [
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ox).unwrap(),
            Pet::try_from(PetName::Tiger).unwrap(),
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(50);
        team
    });
    let mut enemy_team = black_box({
        let pets = [
            Pet::try_from(PetName::Sheep).unwrap(),
            Pet::try_from(PetName::Tiger).unwrap(),
        ];
        let mut team = Team::new(&pets, 5).unwrap();
        team.set_seed(50);
        team
    });
    c.bench_function("sapai_example", |b| {
        b.iter(|| {
            let mut outcome = team.fight(&mut enemy_team);
            while let TeamFightOutcome::None = outcome {
                outcome = team.fight(&mut enemy_team);
            }
        })
    });
}

criterion_group!(benches, bench_blowfish_rally, bench_sapai);
criterion_main!(benches);
