use application::services::strategy_planner::StrategyPlanner;
use domain::models::intent::{Asset, Intent, LendingType};

fn main() {
    let planner = StrategyPlanner::new();

    let intent = Intent::Lend {
        asset: Asset::Usdc,
        amount: 1000_000000, // 1000 USDC (6 decimals)
        protocol: LendingType::Aave,
    };

    match planner.plan(&intent) {
        Ok(plan) => {
            println!("Plan created:");
            println!("Description: {}", plan.description());
            println!("Protocol: {:?}", plan.protocol());
            println!("Steps: {}", plan.step_count());
            println!("Gas: {:?}", plan.gas_estimation());
            println!("\nSteps:");
            for (i, step) in plan.steps().iter().enumerate() {
                println!("  {}. {:?}", i + 1, step);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
