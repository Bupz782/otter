use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub enum Metric {
    Yield,
    Price,
    GasCost,
    Volume,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Comparator {
    GreaterThan,
    LessThan,
    EqualTo,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Condition {
    Comparison {
        metric: Metric,
        comparator: Comparator,
        value: u128,
    },
}

impl Condition {
    /// Return the metric being compared.
    pub fn metric(&self) -> &Metric {
        match self {
            Condition::Comparison { metric, .. } => metric,
        }
    }
}
