INSERT INTO pets (
    name, tier, attack, health, pack,
    effect_trigger, effect, effect_atk, effect_health, n_triggers, temp_effect,
    lvl, cost
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
ON CONFLICT(name, pack, lvl) DO UPDATE SET
    tier = ?2,
    attack = ?3,
    health = ?4,
    effect_trigger = ?6,
    effect = ?7,
    effect_atk = ?8,
    effect_health = ?9,
    n_triggers = ?10,
    temp_effect = ?11
WHERE
    tier != ?2 OR
    attack != ?3 OR
    health != ?4 OR
    effect_trigger != ?6 OR
    effect != ?7
;
