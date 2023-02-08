# SuperAutoTest
[![Release](https://img.shields.io/github/v/release/koisland/SuperAutoTest)]()
[![CI](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml/badge.svg)](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/koisland/SuperAutoTest/branch/main/graph/badge.svg?token=0HTPI2EF7T)](https://codecov.io/gh/koisland/SuperAutoTest)

<img src="docs/images/turtle_crystal_ball.png" width="40%" />

Database and testing framework for Super Auto Pets.

Game information is queried from the [Super Auto Pets Fandom wiki](https://superautopets.fandom.com/wiki) page and stored in a `SQLite3` database.

---

## Usage
Run the `sapdb.exe` in `./bin`.
```bash
./bin/sapdb.exe run
```

This will setup a server locally at [127.0.0.1:8000](http://127.0.0.1:8000)

### Database
From here, you can query pets/tokens (`pet/`) by the following parameters:
1. `name`
    * Name of pet.
2. `level`
    * Level of pet. Alters the effect.
3. `tier`
    * Tier of pet.
4. `pack`
    * Pack pet belongs to.
    * Refer to the SuperAutoPets Fandom wiki page on [Pets](https://superautopets.fandom.com/wiki/Pets) for more information.
5. `effect_trigger`
    * Effect trigger for pet.

Or foods (`/food`) with the following parameters:
1. `name`
    * Name of food.
2. `tier`
    * Tier of food.
3. `pack`
    * Pack food belongs to.

### Testing
Submit two teams to simulate a battle.
* Currently, only pets from the `Turtle` pack are supported.
* To check which ones are allowed:
  * `SQL`
    ```bash
    ./bin/sapdb.exe init && sqlite3 sap.db
    ```
    ```sql
    SELECT DISTINCT name FROM pets WHERE pack='Turtle';
    ```
  * `API`
    ```bash
    curl http://127.0.0.1:8000/pet?pack=Turtle | jq '.[].name' | uniq
    ```


Build your team of pets. A maximum of five are allowed.
* View examples at `docs/examples/`.
```json
{
    "friends": {
        "name": "self",
        "p1": {
            "name": "Ant",
            "attack": 2,
            "health": 1,
            "level": 1
        }
    },
    "enemies": {
        "name": "enemy",
        "p1": {
            "name": "Ant",
            "attack": 2,
            "health": 1,
            "level": 1
        }
    }

}
```

Then submit them to the `battle/` endpoint.
```bash
curl -X POST http://127.0.0.1:8000/battle  -H "Content-Type: application/json" -d @docs/examples/input_win.json
```

Where the output `JSON` shows the outcome of the battle.
```json
{
  "winner": null,
  "friends": [...],
  "friends_fainted": [...],
  "enemies": [...],
  "enemies_fainted": [...],
  "n_turns": 5
}
```

---

## API
To see API usage, see [`docs/README.md`](docs/README.md)
* WIP

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

---

## Examples

To get all **level 2** pets named **'Sloth'** from the **'Turtle' pack**.
```bash
curl http://127.0.0.1:8000/pet?level=2&pack=Turtle&name=Sloth
```

Output:
```json
[
  {
    "name": "Sloth",
    "tier": 1,
    "attack": 1,
    "health": 1,
    "pack": "Turtle",
    "effect_trigger": "None",
    "effect": "Sloth has no special ability. Is kind of lame combat-wise. But he truly believes in you!",
    "effect_atk": 0,
    "effect_health": 0,
    "n_triggers": 1,
    "temp_effect": false,
    "lvl": 2
  }
]
```

To get all **tier 3** foods from the **Star pack**.
```bash
curl http://127.0.0.1:8000/food?tier=3&pack=Star
```

```json
[
  {
    "name": "Pineapple",
    "tier": 3,
    "effect": "Give one pet Pineapple. Ability deals +2 damage",
    "pack": "Star",
    "holdable": true,
    "single_use": false,
    "end_of_battle": false,
    "random": false,
    "n_targets": 1,
    "effect_atk": 2,
    "effect_health": 0,
    "turn_effect": false
  },
  {
    "name": "Cucumber",
    "tier": 3,
    "effect": "Give one pet Cucumber. Gain +1 health at end of turn",
    "pack": "Star",
    "holdable": true,
    "single_use": false,
    "end_of_battle": false,
    "random": false,
    "n_targets": 1,
    "effect_atk": 0,
    "effect_health": 1,
    "turn_effect": true
  }
]
```

### Troubleshooting
Check the logs saved to `~/logs` to debug any issues.

### TODO:
* Add serialization of team and pets.
* Migrate to sqlx
* Deploy to crates.io
* Fix rocket app.
* Expand database fields for unique ability types (summon atk/health, summon percentage, etc.).
* Rework Statistics struct math operations to more consistent.

### Sources
* https://superautopets.fandom.com/wiki
* https://emoji.supply/kitchen/
* https://github.com/manny405/sapai
