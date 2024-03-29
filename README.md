# SAPTest
[![](https://img.shields.io/crates/v/saptest)](https://crates.io/crates/saptest)
[![](https://img.shields.io/docsrs/saptest/latest?color=blue)](https://docs.rs/saptest/latest/saptest/)
[![CI](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml/badge.svg)](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/koisland/SuperAutoTest/branch/main/graph/badge.svg?token=0HTPI2EF7T)](https://codecov.io/gh/koisland/SuperAutoTest)

<img src="docs/images/turtle_crystal_ball.png" width="40%" />

Database and testing framework for Super Auto Pets.

Game information is queried from the [Super Auto Pets wiki](https://superautopets.wiki.gg/wiki/Super_Auto_Pets_Wiki) page and stored in a `SQLite` database.
* [x] [Turtle](https://superautopets.fandom.com/wiki/Turtle_Pack)
* [x] [Puppy](https://superautopets.fandom.com/wiki/Puppy_Pack)
* [x] [Weekly](https://superautopets.fandom.com/wiki/Weekly_Pack)
* [x] [Star](https://superautopets.fandom.com/wiki/Star_Pack)
* [ ] Golden

---

## Usage
Add it to a project with `cargo`.
```bash
cargo add saptest
```

### Teams
Build a `Team` and simulate battles between them.

Then visualize the results in `.dot` format!
```rust
use saptest::{
    Pet, PetName, Food, FoodName,
    Team, TeamCombat, Position, create_battle_digraph
};
// Create a team.
let mut team = Team::new(
    &vec![Some(Pet::try_from(PetName::Ant).unwrap()); 5],
    5
).unwrap();
let mut enemy_team = team.clone();

// Set a seed for a team.
team.set_seed(Some(25));

// Give food to pets.
team.set_item(&Position::First, Food::try_from(FoodName::Garlic).ok()).unwrap();
enemy_team.set_item(&Position::First, Food::try_from(FoodName::Garlic).ok()).unwrap();

// And fight!
team.fight(&mut enemy_team).unwrap();

// Create a graph of the fight.
println!("{}", create_battle_digraph(&team, false));
```

```ignore
digraph {
    rankdir=LR
    node [shape=box, style="rounded, filled", fontname="Arial"]
    edge [fontname="Arial"]
    0 [ label = "Ant_0 - The Fragile Truckers_copy" ]
    1 [ label = "Ant_0 - The Fragile Truckers", fillcolor = "yellow" ]
    2 [ label = "Ant_3 - The Fragile Truckers", fillcolor = "yellow" ]
    3 [ label = "Ant_4 - The Fragile Truckers_copy" ]
    0 -> 1 [ label = "(Attack, Damage (0, 2), Phase: 1)" ]
    1 -> 0 [ label = "(Attack, Damage (0, 2), Phase: 1)" ]
    1 -> 2 [ label = "(Faint, Add (1, 1), Phase: 1)" ]
    0 -> 3 [ label = "(Faint, Add (1, 1), Phase: 1)" ]
}
```

Graphs can be as simple as the example above... or extremely complex.

<p float="left">
    <img align="middle" src="docs/images/ants.svg" width="40%">
    <img align="middle" src="docs/images/blowfish_5.svg" width="30%">
</p>

> Using [Graphviz Online](https://dreampuf.github.io/GraphvizOnline/).

### Shops
Add shop functionality to a `Team` and roll, freeze, buy/sell pets and foods.
```rust
use saptest::{
    Shop, ShopItem, TeamShopping, Team,
    Position, Entity, EntityName, Food, FoodName,
    db::pack::Pack
};

// All teams are constructed with a shop at tier 1.
let mut team = Team::default();

// All shop functionality is supported.
team.set_shop_seed(Some(1212))
    .set_shop_packs(&[Pack::Turtle])
    .open_shop().unwrap()
    .buy(
        &Position::First, // From first.
        &Entity::Pet, // Pet
        &Position::First // To first position, merging if possible.
    ).unwrap()
    .move_pets(
        &Position::First, // First pet.
        &Position::Relative(-2), // To pet 2 slots behind. Otherwise, ignore.
        true // And merge them if possible.
    ).unwrap()
    .sell(&Position::First).unwrap()
    .freeze_shop(&Position::Last, &Entity::Pet).unwrap()
    .roll_shop().unwrap()
    .close_shop().unwrap();

// Shops can be built separately and can replace a team's shop.
let mut tier_5_shop = Shop::new(3, Some(42)).unwrap();
let weakness = ShopItem::new(
    Food::try_from(FoodName::Weak).unwrap()
);
tier_5_shop.add_item(weakness).unwrap();
team.replace_shop(tier_5_shop).unwrap();
```

### Pets
Build custom `Pet`s and `Effect`s.
```rust
use saptest::{
    Pet, PetName, PetCombat,
    Food, FoodName,
    Entity, Position, Effect, Statistics,
    effects::{
        trigger::TRIGGER_START_BATTLE,
        actions::GainType,
        state::Target,
        actions::Action
    }
};
// Create known pets.
let mut pet = Pet::try_from(PetName::Ant).unwrap();

// Or custom pets and effects.
let custom_effect = Effect::new(
    TRIGGER_START_BATTLE, // Effect trigger
    Target::Friend, // Target
    Position::Adjacent, // Positions
    Action::Gain(GainType::DefaultItem(FoodName::Melon)), // Action
    Some(1), // Number of uses.
    false, // Is temporary.
);
let mut custom_pet = Pet::custom(
    "MelonBear",
    Statistics::new(50, 50).unwrap(),
    &[custom_effect],
);

// Fight two pets individually as well.
// Note: Effects don't activate here.
pet.attack(&mut custom_pet);
```

### Logging
Enable logging with a crate like [`simple_logger`](https://docs.rs/simple_logger/latest/simple_logger/#).

### Config
To configure the global `SapDB`'s startup, create a `.saptest.toml` file in the root of your project.
* Specify page versions for pets, foods, and tokens to query.
* Toggle recurring updates on startup.
* Set database filename.

```toml no_run
[database]
# https://superautopets.wiki.gg/index.php?title=Pets&oldid=4634
pets_version = 4634
filename = "./sap.db"
update_on_startup = false
```

Graph building can also be disabled in `[general]` with `build_graph`.
```toml
[general]
build_graph = false
```
---

## Benchmarks
Benchmarks for `saptest` are located in `benches/battle_benchmarks.rs` and run using the [`criterion`](https://docs.rs/crate/criterion/latest) crate.
* Compared against [`sapai`](https://github.com/manny405/sapai#battles), a Super Auto Pets testing framework written in Python.
* Both tests were run on an AMD Ryzen 5 5600 6-Core Processor @ 3.50 GHz.

```bash
# saptest
git clone git@github.com:koisland/SuperAutoTest.git --depth 1
cargo add cargo-tarpaulin
cargo bench && open target/criterion/sapai_example/report/index.html
```

```bash
# sapai
cd benches/
git clone https://github.com/manny405/sapai.git && cd sapai
python setup.py install
# Then run `battle_benchmarks_sapai.ipynb`.
```

### saptest (No Graphs)
* TODO

### saptest (Graphs)
* TODO

### sapai
* **4.29 ms ± 51.8 µs** per loop (mean ± std. dev. of 7 runs, **100 loops each**)

---

## TODO:
* Add trait for randomly generating teams.
* Add method to iterate through record fields/fieldnames and build macro to construct SQL statements.
    * Manually adding fields is tedious and error-prone.
* Build lexer to parse raw effect text into `Effect` struct.
* Add feature flags for each pack.
* Improve interface.
    * Function arguments with impls like `AsRef` and `Into/TryInto`. ex. `Shop.add_item`
    * Type state pattern with `Shop<Open/Close>`
---

## Sources
* https://superautopets.fandom.com/wiki
* https://emoji.supply/kitchen/
* https://github.com/manny405/sapai
* ["ADVANCED Mechanics in Super Auto Pets (Guide)"](https://www.youtube.com/watch?v=NSqjuA32AoA) by Blueberry Pieper
