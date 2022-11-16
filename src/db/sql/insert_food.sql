INSERT INTO foods (name, tier, effect, pack)
VALUES (?1, ?2, ?3, ?4) 
ON CONFLICT(name, pack) DO UPDATE SET 
    tier = ?2,
    effect = ?3
WHERE 
    tier != ?2 OR 
    effect != ?3
;