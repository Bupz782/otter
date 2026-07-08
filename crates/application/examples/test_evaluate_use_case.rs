use application::use_cases::EvaluateConditionUseCase;
use domain::models::condition::{Comparator, Condition, Metric};
use domain::models::intent::Asset;
use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};

/// Simple in-memory oracle for this example.
struct ExampleOracle;

impl PriceOraclePort for ExampleOracle {
    fn fetch(&self, metric: &Metric, _asset: Option<&Asset>) -> Result<u128, OracleError> {
        match metric {
            Metric::Yield => Ok(5),             // 5%
            Metric::Price => Ok(2_500_000_000), // $2,500
            Metric::GasCost => Ok(25),          // 25 gwei
            Metric::Volume => Ok(1_000_000),    // 1M units
        }
    }
}

fn main() {
    let use_case = EvaluateConditionUseCase::new(ExampleOracle);

    let condition = Condition::Comparison {
        metric: Metric::Yield,
        comparator: Comparator::GreaterThan,
        value: 3,
    };

    println!("Evaluating condition: Yield > 3%");

    if use_case.can_evaluate(&condition, Some(&Asset::Usdc)) {
        println!("Can evaluate (oracle available)");

        match use_case.execute(&condition, Some(&Asset::Usdc)) {
            Ok(result) => {
                println!("Result: {}", if result { "TRUE" } else { "FALSE" });
            }
            Err(e) => {
                println!("Evaluation error: {:?}", e);
            }
        }
    } else {
        println!("Cannot evaluate (oracle not available)");
    }

    println!("\n--- Testing other metrics ---");

    let metrics = vec![
        ("Price > 1000", Metric::Price, 1000),
        ("GasCost < 50", Metric::GasCost, 50),
        ("Volume >= 1000000", Metric::Volume, 1_000_000),
    ];

    for (desc, metric, value) in metrics {
        let cond = Condition::Comparison {
            metric,
            comparator: Comparator::GreaterThan,
            value,
        };
        let asset = Asset::Usdc;
        let can_eval = use_case.can_evaluate(&cond, Some(&asset));
        println!("{}: can_evaluate = {}", desc, can_eval);
        if can_eval && let Ok(result) = use_case.execute(&cond, Some(&asset)) {
            println!("  result: {}", if result { "TRUE" } else { "FALSE" });
        }
    }
}
