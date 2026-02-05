use domain::models::{
    execution_plan::{Address, ExecutionPlan, ExecutionStep},
    intent::{Asset, DexType, Intent, LendingType, Protocol},
};

pub struct StrategyPlanner;

#[derive(Debug)]
pub enum PlannerError {
    UnsupportedProtocol(String),
    InvalidAmount,
}

impl StrategyPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(&self, intent: &Intent) -> Result<ExecutionPlan, PlannerError> {
        match intent {
            Intent::Lend {
                asset,
                amount,
                protocol,
            } => self.plan_lend(asset, *amount, protocol),
            Intent::Swap {
                from_asset,
                to_asset,
                amount,
                protocol,
            } => self.plan_swap(from_asset, to_asset, *amount, protocol),
            Intent::Borrow {
                asset,
                amount,
                protocol,
            } => {
                todo!("US-038")
            }
            Intent::Stake {
                asset,
                amount,
                protocol,
            } => {
                todo!("US-039")
            }
        }
    }

    fn plan_lend(
        &self,
        asset: &Asset,
        amount: u128,
        protocol: &LendingType,
    ) -> Result<ExecutionPlan, PlannerError> {
        let pool_address = self.get_lending_pool_address(protocol);
        let protocol_enum = Protocol::Lending(protocol.clone());

        let approve_step = ExecutionStep::Approve {
            asset: asset.clone(),
            spender: pool_address,
            amount,
        };

        let supply_step = ExecutionStep::Supply {
            asset: asset.clone(),
            amount,
            protocol: protocol_enum.clone(),
        };

        let description = format!("Lend {:?} via {:?}", asset, protocol);

        let plan = ExecutionPlan::new(protocol_enum, description)
            .with_steps(vec![approve_step, supply_step])
            .with_gas_estimation(150_000);

        Ok(plan)
    }

    fn get_lending_pool_address(&self, protocol: &LendingType) -> Address {
        match protocol {
            LendingType::Aave => "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(), // Aave V3 Sepolia
            LendingType::Compound => "0xCOMPOUND_POOL".to_string(), // Placeholder
        }
    }

    fn plan_swap(
        &self,
        from_asset: &Asset,
        to_asset: &Asset,
        amount: u128,
        protocol: &DexType,
    ) -> Result<ExecutionPlan, PlannerError> {
        let pool_address = self.get_dex_router_address(protocol);
        let protocol_enum = Protocol::Dex(protocol.clone());

        let approve_step = ExecutionStep::Approve {
            asset: from_asset.clone(),
            spender: pool_address,
            amount,
        };
        let min_amount_out = amount * 99 / 100; // Simple 1% slippage
        let swap_step = ExecutionStep::SwapExactTokens {
            from_asset: from_asset.clone(),
            to_asset: to_asset.clone(),
            amount_in: amount,
            min_amount_out: min_amount_out,
            protocol: protocol_enum.clone(),
        };

        let description = format!(
            "Swap {:?} To {:?} Via {:?} ",
            from_asset, to_asset, protocol
        );

        let plan = ExecutionPlan::new(protocol_enum, description)
            .with_steps(vec![approve_step, swap_step])
            .with_gas_estimation(150_000);
        return Ok(plan);
    }

    fn get_dex_router_address(&self, protocol: &DexType) -> Address {
        match protocol {
            DexType::Uniswap => "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
            DexType::Sushiswap => "0xSUSHI_ROUTER".to_string(),
            DexType::Balancer => "0xBALANCER_ROUTER".to_string(),
        }
    }
}
