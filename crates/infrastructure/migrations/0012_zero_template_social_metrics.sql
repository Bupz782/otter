-- A2 follow-up: protocol strategy templates seeded before the fix carry
-- fabricated social metrics (copies, fork_count, total_volume, apy). These
-- counters are measured, never seeded — zero them for protocol templates
-- (creator_address IS NULL). User-created strategies keep their real counts.
UPDATE strategies
SET copies = 0, fork_count = 0, total_volume = 0, apy = 0
WHERE creator_address IS NULL;
