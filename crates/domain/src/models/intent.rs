use crate::models::condition::Condition;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidAmount(String),
    InvalidAsset(String),
    InvalidProtocol(String),
    MissingField(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Asset {
    Eth,
    Dai,
    Usdc,
    Wbtc,
    Link,
    Sol,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum DexType {
    Uniswap,
    Sushiswap,
    Balancer,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum LendingType {
    Aave,
    Compound,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Protocol {
    Dex(DexType),
    Lending(LendingType),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ConditionalIntent {
    pub intent: Intent,
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Intent {
    Swap {
        from_asset: Asset,
        to_asset: Asset,
        amount: u128,
        protocol: DexType,
    },
    Stake {
        asset: Asset,
        amount: u128,
        protocol: LendingType,
    },
    Borrow {
        asset: Asset,
        amount: u128,
        collateral: Asset,
        collateral_amount: u128,
        protocol: LendingType,
    },
    Lend {
        asset: Asset,
        amount: u128,
        protocol: LendingType,
    },
    Composite {
        intents: Vec<Intent>,
    },
}

impl Asset {
    pub fn decimals(&self) -> u8 {
        match self {
            Asset::Eth => 18,
            Asset::Dai => 18,
            Asset::Usdc => 6,
            Asset::Wbtc => 8,
            Asset::Link => 18,
            Asset::Sol => 9,
        }
    }
    pub fn parse_amount(&self, amount_str: &str) -> Option<u128> {
        let parts: Vec<&str> = amount_str.split('.').collect();

        match parts.len() {
            1 => {
                let integer = parts[0].parse::<u128>().ok()?;
                Some(integer * 10u128.pow(self.decimals() as u32))
            }
            2 => {
                let integer_part = parts[0];
                let decimal_part = parts[1];
                let decimal_places = decimal_part.len() as u32;

                let full_number = format!(
                    "{}{}{}",
                    integer_part,
                    decimal_part,
                    "0".repeat((self.decimals() as u32 - decimal_places) as usize)
                );

                full_number.parse::<u128>().ok()
            }
            _ => None,
        }
    }
    pub fn format_amount(&self, amount: u128) -> String {
        let decimals = self.decimals() as u32;
        let divisor = 10u128.pow(decimals);
        let integer_part = amount / divisor;
        let decimal_part = amount % divisor;

        if decimal_part == 0 {
            format!("{}", integer_part)
        } else {
            let decimal_str = format!("{:0>width$}", decimal_part, width = decimals as usize);
            let trimmed = decimal_str.trim_end_matches('0');
            format!("{}.{}", integer_part, trimmed)
        }
    }
}

impl Intent {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Intent::Swap {
                from_asset,
                to_asset,
                amount,
                ..
            } => {
                if *amount == 0 {
                    return Err(ValidationError::InvalidAmount(
                        "Amount must be greater than 0".to_string(),
                    ));
                }
                if from_asset == to_asset {
                    return Err(ValidationError::InvalidAsset(
                        "Cannot swap same asset".to_string(),
                    ));
                }
                Ok(())
            }
            Intent::Lend { amount, .. } => {
                if *amount == 0 {
                    return Err(ValidationError::InvalidAmount(
                        "Amount must be greater than 0".to_string(),
                    ));
                }
                Ok(())
            }
            Intent::Borrow { amount, .. } => {
                if *amount == 0 {
                    return Err(ValidationError::InvalidAmount(
                        "Amount must be greater than 0".to_string(),
                    ));
                }
                Ok(())
            }
            Intent::Stake { amount, .. } => {
                if *amount == 0 {
                    return Err(ValidationError::InvalidAmount(
                        "Amount must be greater than 0".to_string(),
                    ));
                }
                Ok(())
            }
            Intent::Composite { intents } => {
                if intents.is_empty() {
                    return Err(ValidationError::InvalidAmount(
                        "Composite intent must have at least one intent".to_string(),
                    ));
                }
                for intent in intents {
                    intent.validate()?;
                }
                Ok(())
            }
        }
    }
}
