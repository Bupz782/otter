use domain::models::condition::{Comparator, Condition};
use domain::models::intent::Asset;
use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};

pub struct EvaluateConditionUseCase<O> {
    oracle: O,
}

#[derive(Debug)]
pub enum EvaluationError {
    OracleUnavailable(String),
    InvalidMetric(String),
    EvaluationFailed(String),
}

impl From<OracleError> for EvaluationError {
    fn from(err: OracleError) -> Self {
        EvaluationError::OracleUnavailable(err.to_string())
    }
}

impl<O> EvaluateConditionUseCase<O>
where
    O: PriceOraclePort,
{
    pub fn new(oracle: O) -> Self {
        Self { oracle }
    }

    /// Evaluate a condition by fetching the current metric value from the
    /// configured oracle and comparing it against the threshold.
    pub fn execute(
        &self,
        condition: &Condition,
        asset: Option<&Asset>,
    ) -> Result<bool, EvaluationError> {
        match condition {
            Condition::Comparison {
                metric,
                comparator,
                value,
            } => {
                let current = self.oracle.fetch(metric, asset)?;
                Ok(compare(current, *value, comparator))
            }
        }
    }

    /// Check if the condition's metric can be evaluated by this oracle.
    pub fn can_evaluate(&self, condition: &Condition, asset: Option<&Asset>) -> bool {
        match condition {
            Condition::Comparison { metric, .. } => self.oracle.fetch(metric, asset).is_ok(),
        }
    }
}

fn compare(current: u128, threshold: u128, comparator: &Comparator) -> bool {
    match comparator {
        Comparator::GreaterThan => current > threshold,
        Comparator::LessThan => current < threshold,
        Comparator::EqualTo => current == threshold,
        Comparator::GreaterThanOrEqualTo => current >= threshold,
        Comparator::LessThanOrEqualTo => current <= threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::condition::{Comparator, Condition, Metric};
    use domain::models::intent::Asset;
    use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};

    struct FixedOracle {
        value: u128,
    }

    impl PriceOraclePort for FixedOracle {
        fn fetch(&self, _metric: &Metric, _asset: Option<&Asset>) -> Result<u128, OracleError> {
            Ok(self.value)
        }
    }

    #[test]
    fn evaluates_greater_than_true() {
        let oracle = FixedOracle { value: 5 };
        let use_case = EvaluateConditionUseCase::new(oracle);
        let condition = Condition::Comparison {
            metric: Metric::Yield,
            comparator: Comparator::GreaterThan,
            value: 3,
        };
        assert!(use_case.execute(&condition, Some(&Asset::Usdc)).unwrap());
    }

    #[test]
    fn evaluates_greater_than_false() {
        let oracle = FixedOracle { value: 2 };
        let use_case = EvaluateConditionUseCase::new(oracle);
        let condition = Condition::Comparison {
            metric: Metric::Yield,
            comparator: Comparator::GreaterThan,
            value: 3,
        };
        assert!(!use_case.execute(&condition, Some(&Asset::Usdc)).unwrap());
    }

    #[test]
    fn evaluates_equal_to() {
        let oracle = FixedOracle { value: 100 };
        let use_case = EvaluateConditionUseCase::new(oracle);
        let condition = Condition::Comparison {
            metric: Metric::Price,
            comparator: Comparator::EqualTo,
            value: 100,
        };
        assert!(use_case.execute(&condition, Some(&Asset::Eth)).unwrap());
    }
}
