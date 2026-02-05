use crate::models::intent::{Asset, Protocol};

pub type Address = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    steps: Vec<ExecutionStep>,
    protocol: Protocol,
    gas_estimation: Option<u64>,
    description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStep {
    Approve {
        asset: Asset,
        spender: Address,
        amount: u128,
    },
    Supply {
        asset: Asset,
        amount: u128,
        protocol: Protocol,
    },
    Borrow {
        asset: Asset,
        amount: u128,
        protocol: Protocol,
    },
    Repay {
        asset: Asset,
        amount: u128,
        protocol: Protocol,
    },
    Withdraw {
        asset: Asset,
        amount: u128,
        protocol: Protocol,
    },
    SwapExactTokens {
        from_asset: Asset,
        to_asset: Asset,
        amount_in: u128,
        min_amount_out: u128,
        protocol: Protocol,
    },
    Stake {
        asset: Asset,
        amount: u128,
        protocol: Protocol,
    },
    Unstake {
        asset: Asset,
        amount: u128,
        protocol: Protocol,
    },
}

impl ExecutionPlan {
    pub fn new(protocol: Protocol, description: impl Into<String>) -> Self {
        Self {
            steps: Vec::new(),
            protocol,
            gas_estimation: None,
            description: description.into(),
        }
    }

    pub fn with_steps(mut self, steps: Vec<ExecutionStep>) -> Self {
        self.steps = steps;
        self
    }

    pub fn with_gas_estimation(mut self, gas: u64) -> Self {
        self.gas_estimation = Some(gas);
        self
    }

    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }

    pub fn steps(&self) -> &[ExecutionStep] {
        &self.steps
    }

    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }

    pub fn gas_estimation(&self) -> Option<u64> {
        self.gas_estimation
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}
