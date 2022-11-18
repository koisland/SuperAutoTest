INSERT INTO pets (name, tier, attack, health, pack, effect_trigger, effect, lvl)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
ON CONFLICT(name, pack, lvl) DO UPDATE SET 
    tier = ?2,
    attack = ?3,
    health = ?4,
    effect_trigger = ?6,
    effect = ?7
WHERE
    tier != ?2 OR
    attack != ?3 OR
    health != ?4 OR
    effect_trigger != ?6 OR
    effect != ?7
;