use domain::models::{
    condition::Condition,
    execution_plan::{Address, ExecutionPlan, ExecutionStep},
    intent::{Asset, ConditionalIntent, DexType, Intent, LendingType, Protocol},
};

pub struct StrategyPlanner;

#[derive(Debug)]
pub enum PlannerError {
    UnsupportedProtocol(String),
    InvalidAmount,
    InvalidCondition(String),
    EmptyComposite,
    InsufficientCollateral,
    InvalidSequence(String),
}

impl Default for StrategyPlanner {
    fn default() -> Self {
        Self::new()
    }
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
                collateral,
                collateral_amount,
                protocol,
            } => self.plan_borrow(asset, *amount, collateral, *collateral_amount, protocol),
            Intent::Stake {
                asset,
                amount,
                protocol,
            } => self.plan_stake(asset, *amount, protocol),
            Intent::Composite { intents } => self.plan_composite(intents),
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
            .with_calculated_gas();

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
            min_amount_out,
            protocol: protocol_enum.clone(),
        };

        let description = format!(
            "Swap {:?} To {:?} Via {:?} ",
            from_asset, to_asset, protocol
        );

        let plan = ExecutionPlan::new(protocol_enum, description)
            .with_steps(vec![approve_step, swap_step])
            .with_calculated_gas();
        Ok(plan)
    }

    fn get_dex_router_address(&self, protocol: &DexType) -> Address {
        match protocol {
            DexType::Uniswap => "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string(),
            DexType::Sushiswap => "0xSUSHI_ROUTER".to_string(),
            DexType::Balancer => "0xBALANCER_ROUTER".to_string(),
        }
    }

    fn plan_stake(
        &self,
        asset: &Asset,
        amount: u128,
        protocol: &LendingType,
    ) -> Result<ExecutionPlan, PlannerError> {
        let staking_address = self.get_staking_address(protocol);
        let protocol_enum = Protocol::Lending(protocol.clone());

        let approve_step = ExecutionStep::Approve {
            asset: asset.clone(),
            spender: staking_address,
            amount,
        };

        let stake_step = ExecutionStep::Stake {
            asset: asset.clone(),
            amount,
            protocol: protocol_enum.clone(),
        };

        let description = format!("Stake {:?} via {:?}", asset, protocol);

        let plan = ExecutionPlan::new(protocol_enum, description)
            .with_steps(vec![approve_step, stake_step])
            .with_calculated_gas();

        Ok(plan)
    }

    fn get_staking_address(&self, protocol: &LendingType) -> Address {
        match protocol {
            LendingType::Aave => "0x4da27a545c0c5B758a6BA100e3a049001de870f5".to_string(), // stkAAVE
            LendingType::Compound => "0xCOMPOUND_STAKE".to_string(),
        }
    }

    fn plan_borrow(
        &self,
        asset: &Asset,
        amount: u128,
        collateral: &Asset,
        collateral_amount: u128,
        protocol: &LendingType,
    ) -> Result<ExecutionPlan, PlannerError> {
        let pool_address = self.get_lending_pool_address(protocol);
        let protocol_enum = Protocol::Lending(protocol.clone());

        let approve_collateral = ExecutionStep::Approve {
            asset: collateral.clone(),
            spender: pool_address.clone(),
            amount: collateral_amount,
        };

        let supply_collateral = ExecutionStep::Supply {
            asset: collateral.clone(),
            amount: collateral_amount,
            protocol: protocol_enum.clone(),
        };

        let borrow_step = ExecutionStep::Borrow {
            asset: asset.clone(),
            amount,
            protocol: protocol_enum.clone(),
        };

        let description = format!(
            "Borrow {:?} via {:?} (collateral: {:?})",
            asset, protocol, collateral
        );

        let plan = ExecutionPlan::new(protocol_enum, description)
            .with_steps(vec![approve_collateral, supply_collateral, borrow_step])
            .with_calculated_gas();

        Ok(plan)
    }

    fn plan_composite(&self, intents: &[Intent]) -> Result<ExecutionPlan, PlannerError> {
        if intents.is_empty() {
            return Err(PlannerError::EmptyComposite);
        }

        let mut all_steps: Vec<ExecutionStep> = Vec::new();
        let mut descriptions: Vec<String> = Vec::new();

        let first_protocol = self.extract_protocol(&intents[0]);

        for intent in intents {
            let plan = self.plan(intent)?;
            all_steps.extend(plan.steps().to_vec());
            descriptions.push(plan.description().to_string());
        }

        let description = format!("Composite: {}", descriptions.join(" → "));

        let plan = ExecutionPlan::new(first_protocol, description)
            .with_steps(all_steps)
            .with_calculated_gas();

        Ok(plan)
    }

    fn extract_protocol(&self, intent: &Intent) -> Protocol {
        match intent {
            Intent::Lend { protocol, .. } => Protocol::Lending(protocol.clone()),
            Intent::Borrow { protocol, .. } => Protocol::Lending(protocol.clone()),
            Intent::Stake { protocol, .. } => Protocol::Lending(protocol.clone()),
            Intent::Swap { protocol, .. } => Protocol::Dex(protocol.clone()),
            Intent::Composite { intents } => {
                if let Some(first) = intents.first() {
                    self.extract_protocol(first)
                } else {
                    Protocol::Lending(LendingType::Aave) // fallback
                }
            }
        }
    }

    pub fn plan_conditional(
        &self,
        conditional: &ConditionalIntent,
    ) -> Result<ExecutionPlan, PlannerError> {
        if let Some(condition) = &conditional.condition {
            self.validate_condition(condition)?;
        }

        self.plan(&conditional.intent)
    }

    fn validate_condition(&self, condition: &Condition) -> Result<(), PlannerError> {
        match condition {
            Condition::Comparison { value, .. } => {
                if *value == 0 {
                    return Err(PlannerError::InvalidCondition(
                        "Condition value cannot be zero".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }

    pub fn validate_plan_feasibility(&self, intent: &Intent) -> Result<(), PlannerError> {
        match intent {
            Intent::Borrow {
                amount,
                collateral_amount,
                ..
            } => {
                if *amount == 0 {
                    return Err(PlannerError::InvalidAmount);
                }
                if *collateral_amount == 0 {
                    return Err(PlannerError::InsufficientCollateral);
                }
                Ok(())
            }
            Intent::Swap {
                amount,
                from_asset,
                to_asset,
                ..
            } => {
                if *amount == 0 {
                    return Err(PlannerError::InvalidAmount);
                }
                if from_asset == to_asset {
                    return Err(PlannerError::InvalidSequence(
                        "Cannot swap same asset".to_string(),
                    ));
                }
                Ok(())
            }
            Intent::Lend { amount, .. } | Intent::Stake { amount, .. } => {
                if *amount == 0 {
                    return Err(PlannerError::InvalidAmount);
                }
                Ok(())
            }
            Intent::Composite { intents } => {
                if intents.is_empty() {
                    return Err(PlannerError::EmptyComposite);
                }
                for intent in intents {
                    self.validate_plan_feasibility(intent)?;
                }
                Ok(())
            }
        }
    }
}
