INSERT INTO pets (name, tier, attack, health, pack, effect_trigger, effect_lvl_1, effect_lvl_2, effect_lvl_3)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
ON CONFLICT(name, pack) DO UPDATE SET 
    tier = ?2,
    attack = ?3,
    health = ?4,
    effect_trigger = ?6,
    effect_lvl_1 = ?7,
    effect_lvl_2 = ?8,
    effect_lvl_3 = ?9
WHERE
    tier != ?2 OR
    attack != ?3 OR
    health != ?4 OR
    effect_trigger != ?6 OR
    effect_lvl_1 != ?7 OR
    effect_lvl_2 != ?8 OR
    effect_lvl_3 != ?9 
;