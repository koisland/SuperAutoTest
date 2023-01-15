INSERT INTO foods (
    name, tier, effect, pack,
    holdable, single_use, end_of_battle,
    random, n_targets,
    effect_atk, effect_health,
    turn_effect
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
ON CONFLICT(name, pack) DO UPDATE SET
    tier = ?2,
    effect = ?3,
    pack = ?4,
    holdable = ?5,
    single_use = ?6,
    end_of_battle = ?7,
    random = ?8,
    n_targets = ?9,
    effect_atk = ?10,
    effect_health = ?11,
    turn_effect = ?12
WHERE
    tier != ?2 OR
    effect != ?3
;
