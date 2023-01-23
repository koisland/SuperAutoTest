BEGIN;
CREATE TABLE IF NOT EXISTS pets (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    tier INTEGER NOT NULL,
    attack INTEGER NOT NULL,
    health INTEGER NOT NULL,
    pack TEXT NOT NULL,
    effect_trigger TEXT NOT NULL,
    effect TEXT NOT NULL,
    effect_atk INTEGER NOT NULL,
    effect_health INTEGER NOT NULL,
    n_triggers INTEGER NOT NULL,
    temp_effect BOOLEAN NOT NULL,
    lvl INTEGER NOT NULL,
    cost INTEGER NOT NULL,
    CONSTRAINT unq UNIQUE (name, pack, lvl)
);
CREATE TABLE IF NOT EXISTS foods (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    tier INTEGER NOT NULL,
    effect TEXT NOT NULL,
    pack TEXT NOT NULL,
    holdable BOOLEAN NOT NULL,
    single_use BOOLEAN NOT NULL,
    end_of_battle BOOLEAN NOT NULL,
    random BOOLEAN NOT NULL,
    n_targets INTEGER NOT NULL,
    effect_atk INTEGER NOT NULL,
    effect_health INTEGER NOT NULL,
    turn_effect BOOLEAN NOT NULL,
    cost INTEGER NOT NULL,
    CONSTRAINT unq UNIQUE (name, pack)
);
COMMIT;
