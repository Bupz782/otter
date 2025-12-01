#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Metric {
    Yield,
    Price,
    GasCost,
    Volume,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comparator {
    GreaterThan,
    LessThan,
    EqualTo,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Comparison {
        metric: Metric,
        comparator: Comparator,
        value: u64,
    }
}