# SAPTest
[![](https://img.shields.io/crates/v/saptest)](https://crates.io/crates/saptest)
[![](https://img.shields.io/docsrs/saptest/latest?color=blue)](https://docs.rs/saptest/latest/saptest/)
[![CI](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml/badge.svg)](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/koisland/SuperAutoTest/branch/main/graph/badge.svg?token=0HTPI2EF7T)](https://codecov.io/gh/koisland/SuperAutoTest)

<img src="docs/images/turtle_crystal_ball.png" width="40%" />

Database and testing framework for Super Auto Pets.

Game information is queried from the [Super Auto Pets Fandom wiki](https://superautopets.fandom.com/wiki) page and stored in a `SQLite` database.

---

## Usage

### Teams
Build a `Team` and simulate battles between them.
```rust
use saptest::{Pet, PetName, Food, FoodName, Team, Position};

// Create a team.
let mut team = Team::new(
    &vec![Pet::try_from(PetName::Ant).unwrap(); 5],
    5
).unwrap();
let mut enemy_team = team.clone();

// Set a seed for a team.
team.set_seed(Some(25));

// Give food to pets.
team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());
enemy_team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());

// And fight!
team.fight(&mut enemy_team);
```

### Shops
Add shop functionality to a `Team` and roll, freeze, buy/sell pets and foods.
```rust
use saptest::{
    Shop, ShopItem, TeamShopping, Team,
    Position, Entity, EntityName, FoodName
};

// All teams are constructed with a shop at tier 1.
let mut team = Team::default();

// All shop functionality is supported.
team.set_shop_seed(Some(1212))
    .open_shop().unwrap()
    .buy(&Position::First, &Entity::Pet, &Position::First).unwrap()
    .sell(&Position::First).unwrap()
    .freeze_shop(Position::Last, Entity::Pet).unwrap()
    .roll_shop().unwrap()
    .close_shop().unwrap();

// Shops can be built separately and can replace a team's shop.
let mut tier_5_shop = Shop::new(3, Some(42)).unwrap();
let weakness = ShopItem::new(
    EntityName::Food(FoodName::Weak),
    5
).unwrap();
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
    battle::{
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
    Entity::Pet,
    TRIGGER_START_BATTLE, // Effect trigger
    Target::Friend, // Target
    Position::Adjacent, // Positions
    Action::Gain(GainType::DefaultItem(FoodName::Melon)), // Action
    Some(1), // Number of uses.
    false, // Is temporary.
);
let mut custom_pet = Pet::custom(
    "MelonBear",
    Some("melonbear_1".to_string()),
    Statistics::new(50, 50).unwrap(),
    &[custom_effect],
);

// Fight two pets individually as well.
// Note: Effects don't activate here.
pet.attack(&mut custom_pet);
```

---

## Benchmarks
Benchmarks for `saptest` are located in `benches/battle_benchmarks.rs` and run using the [`criterion`](https://docs.rs/crate/criterion/latest) crate.
* Compared against [`sapai`](https://github.com/manny405/sapai#battles), a Super Auto Pets testing framework written in Python.
* Both tests were run on an AMD Ryzen 5 5600 6-Core Processor @ 3.50 GHz.

```bash
# saptest
cargo bench && open target/criterion/sapai_example/report/index.html
```

```bash
# sapai
cd benches/
git clone https://github.com/manny405/sapai.git && cd sapai
python setup.py install
# Then run `battle_benchmarks_sapai.ipynb`.
```

### saptest
* **166.84 ns ± 1.0369 µs** with **100 measurements**.

![](docs/images//pdf.svg)

### sapai
* **4.29 ms ± 51.8 µs** per loop (mean ± std. dev. of 7 runs, **100 loops each**)

---

## Troubleshooting
To enable verbose logging to troubleshoot issues, enable `log4rs` and use the main log config file.
```rust

fn main() {
  log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

  // Code here.
  ...
}
```

---
## TODO:
* Expand database fields for unique ability types (summon atk/health, summon percentage, etc.).
* Add custom pack parser and reader.
* Add toml config to select which version of the wiki page to use.
* Reorganize Team impl.
  * Move to new trait.
* Add trait for randomly generating teams.
* Create Rust binding for Python.

---
## Sources
* https://superautopets.fandom.com/wiki
* https://emoji.supply/kitchen/
* https://github.com/manny405/sapai
