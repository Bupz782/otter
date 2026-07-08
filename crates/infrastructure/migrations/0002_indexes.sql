CREATE INDEX IF NOT EXISTS idx_intents_state ON intents(state);
CREATE INDEX IF NOT EXISTS idx_intents_user ON intents(user_address);
CREATE INDEX IF NOT EXISTS idx_executions_intent ON executions(intent_id);
CREATE INDEX IF NOT EXISTS idx_delegations_user ON delegations(user_address);

