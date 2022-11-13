BEGIN;
CREATE TABLE IF NOT EXISTS pets (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    tier INTEGER NOT NULL,
    attack INTEGER NOT NULL,
    health INTEGER NOT NULL,
    pack TEXT NOT NULL,
    effect_trigger TEXT NOT NULL,
    effect_lvl_1 TEXT NOT NULL,
    effect_lvl_2 TEXT NOT NULL,
    effect_lvl_3 TEXT NOT NULL,
    CONSTRAINT unq UNIQUE (name, pack)
);
CREATE TABLE IF NOT EXISTS foods (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    tier INTEGER NOT NULL,
    effect TEXT NOT NULL,
    pack TEXT NOT NULL,
    CONSTRAINT unq UNIQUE (name, pack)
);
COMMIT;