use application::services::strategy_planner::StrategyPlanner;
use application::use_cases::PlanExecutionUseCase;
use domain::models::condition::{Comparator, Condition, Metric};
use domain::models::intent::{Asset, ConditionalIntent, Intent, LendingType};

fn main() {
    let planner = StrategyPlanner::new();
    let use_case = PlanExecutionUseCase::new(planner);

    // Test 1: Simple intent
    println!("Test 1: Simple Intent");
    let intent = Intent::Lend {
        asset: Asset::Usdc,
        amount: 1000_000000,
        protocol: LendingType::Aave,
    };

    match use_case.execute(&intent) {
        Ok(plan) => {
            println!("Plan created:");
            println!("  Description: {}", plan.description());
            println!("  Steps: {}", plan.step_count());
            println!("  Gas: {:?}", plan.gas_estimation());
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!("\nTest 2: Conditional Intent");
    let conditional = ConditionalIntent {
        intent: Intent::Lend {
            asset: Asset::Usdc,
            amount: 1000_000000,
            protocol: LendingType::Aave,
        },
        condition: Some(Condition::Comparison {
            metric: Metric::Yield,
            comparator: Comparator::GreaterThan,
            value: 3,
        }),
        network: None,
    };

    match use_case.execute_conditional(&conditional) {
        Ok(plan) => {
            println!("Conditional plan created:");
            println!("  Description: {}", plan.description());
            println!("  Steps: {}", plan.step_count());
            println!("  Gas: {:?}", plan.gas_estimation());
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 3: Composite intent
    println!("\nTest 3: Composite Intent (Lend + Stake)");
    let composite = Intent::Composite {
        intents: vec![
            Intent::Lend {
                asset: Asset::Usdc,
                amount: 1000_000000,
                protocol: LendingType::Aave,
            },
            Intent::Stake {
                asset: Asset::Usdc,
                amount: 1000_000000,
                protocol: LendingType::Aave,
            },
        ],
    };

    match use_case.execute(&composite) {
        Ok(plan) => {
            println!("Composite plan created:");
            println!("  Description: {}", plan.description());
            println!("  Steps: {}", plan.step_count());
            println!("  Gas: {:?}", plan.gas_estimation());
            println!("  Steps detail:");
            for (i, step) in plan.steps().iter().enumerate() {
                println!("    {}. {:?}", i + 1, step);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
