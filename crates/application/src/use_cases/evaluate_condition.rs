use domain::models::condition::{Condition, Metric};

pub struct EvaluateConditionUseCase;

#[derive(Debug)]
pub enum EvaluationError {
    OracleUnavailable(String),
    InvalidMetric(String),
    EvaluationFailed(String),
}

impl EvaluateConditionUseCase {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a condition
    ///
    /// For MVP: Always returns true (stub)
    /// Later: Will fetch real data from oracles/RPC
    pub fn execute(&self, _condition: &Condition) -> Result<bool, EvaluationError> {
        // TODO (Epic 5): Implement real evaluation
        // - Fetch current yield from Aave API
        // - Fetch current price from oracle
        // - Compare with threshold

        // For now: stub that always returns true
        Ok(true)
    }

    /// Check if condition can be evaluated (prerequisites met)
    pub fn can_evaluate(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Comparison { metric, .. } => {
                match metric {
                    // For MVP, we can't evaluate any metrics yet (no oracle)
                    Metric::Yield | Metric::Price | Metric::GasCost | Metric::Volume => false,
                }
            }
        }
    }
}

impl Default for EvaluateConditionUseCase {
    fn default() -> Self {
        Self::new()
    }
}
