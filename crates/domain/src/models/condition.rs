use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Metric {
    Yield,
    Price,
    GasCost,
    Volume,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Comparator {
    GreaterThan,
    LessThan,
    EqualTo,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Condition {
    Comparison {
        metric: Metric,
        comparator: Comparator,
        value: u128,
    },
}
