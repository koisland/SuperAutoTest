# SuperAutoTest
[![CI](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml/badge.svg)](https://github.com/koisland/SuperAutoTest/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/koisland/SuperAutoTest/branch/test_framework/graph/badge.svg?token=0HTPI2EF7T)](https://codecov.io/gh/koisland/SuperAutoTest)

<img src="docs/images/turtle_crystal_ball.png" width="40%" />

Database and testing framework for Super Auto Pets.

Game information is queried from the [Super Auto Pets Fandom wiki](https://superautopets.fandom.com/wiki) page and stored in a `SQLite3` database.

---

## Usage
Run the `sapdb.exe` in `./bin`.
```bash
./bin/sapdb.exe
```

This will setup a server locally at [127.0.0.1:8000](http://127.0.0.1:8000)

### Database
From here, you can query pets by the following parameters:
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

Or foods with the following parameters:
1. `name`
2. `tier`
3. `pack`

### Testing
This is still a WIP.

The general idea is that a user would submit a `JSON` payload of pets for two teams to simulate a battle.

The output `JSON` would detail:
* The winning team.
* Each events in the fight.

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
    "effect": "Give one pet Pineapple. ability deals +2 damage",
    "pack": "Star"
  },
  {
    "name": "Cucumber",
    "tier": 3,
    "effect": "Give one pet Cucumber. Gain +1 health at end of turn",
    "pack": "Star"
  }
]
```

### Troubleshooting
Check the logs saved to `~/logs` to debug any issues.


### Sources
https://superautopets.fandom.com/wiki
https://emoji.supply/kitchen/
