# SuperAutoTest
[![Release](https://img.shields.io/github/v/release/koisland/SuperAutoTest)]()
[![CI](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml/badge.svg)](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/koisland/SuperAutoTest/branch/main/graph/badge.svg?token=0HTPI2EF7T)](https://codecov.io/gh/koisland/SuperAutoTest)

<img src="docs/images/turtle_crystal_ball.png" width="40%" />

Database and testing framework for Super Auto Pets.

Game information is queried from the [Super Auto Pets Fandom wiki](https://superautopets.fandom.com/wiki) page and stored in a `SQLite3` database.

---

## Usage
```rust
use sapt::{SapDB, Pet, PetName, PetCombat, Food, FoodName, Team, Position};

// Initialize the database.
let db = SapDB::new();

// Create pets.
let pet = Pet::try_from(PetName::Ant).unwrap();
let enemy_pet = Pet::try_from(PetName::Ant).unwrap();

// Create a team.
let mut team = Team::new(&vec![pet; 5], 5).unwrap();
let mut enemy_team = Team::new(&vec![enemy_pet; 5], 5).unwrap();

// Give food to pets.
team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());
enemy_team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());

// And fight as a team.
team.fight(&mut enemy_team);
```

---
## Benchmarks
Benchmarks for `sapt` are located in `benches/battle_benchmarks.rs` and run using the [`criterion`](https://docs.rs/crate/criterion/latest) crate.

Compared against [`sapai`](https://github.com/manny405/sapai#battles), a Super Auto Pets testing framework written in Python.

Both tests were run on an AMD Ryzen 5 5600 6-Core Processor @ 3.50 GHz.

```bash
# sapt
cargo bench && open target/criterion/sapai_example/report/index.html
```

```bash
# sapai
cd benches/
git clone https://github.com/manny405/sapai.git && cd sapai
python setup.py install
# Then run `battle_benchmarks_sapai.ipynb`.
```

### sapt
* **166.84 ns ± 1.0369 µs** with **100 measurements**.

![](docs/images//pdf.svg)

### sapai
* **4.29 ms ± 51.8 µs** per loop (mean ± std. dev. of 7 runs, **100 loops each**)


### Troubleshooting
To enable verbose logging to troubleshoot issues, enable `log4rs` and use the main log config file.
```rust
log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();
```

### TODO:
* Add serialization of team and pets.
* Migrate to `sqlx`.
* Deploy to `crates.io`.
* Expand database fields for unique ability types (summon atk/health, summon percentage, etc.).
* Rework Statistics struct math operations to be more consistent.
* Add shops.
  * Consider using the Python package [sapai](https://github.com/manny405/sapai) if shop functionality is required.

### Sources
* https://superautopets.fandom.com/wiki
* https://emoji.supply/kitchen/
* https://github.com/manny405/sapai
