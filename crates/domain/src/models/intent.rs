use crate::models::condition::Condition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asset {
    Eth,
    Dai,
    Usdc,
    Wbtc,
    Link,
    Sol,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DexType {
    Uniswap,
    Sushiswap,
    Balancer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LendingType {
    Aave,
    Compound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Dex(DexType),
    Lending(LendingType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalIntent {
    pub intent: Intent,
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn format_amount(&self, _amount: u128) -> String {
        todo!()
    }
}
