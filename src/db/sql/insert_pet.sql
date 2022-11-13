INSERT OR REPLACE INTO pets (
    name,
    tier,
    attack,
    health,
    pack,
    effect_trigger,
    effect_lvl_1,
    effect_lvl_2,
    effect_lvl_3
)
VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8,
    ?9
);