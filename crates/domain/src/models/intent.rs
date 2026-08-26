use crate::models::condition::Condition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidAmount(String),
    InvalidAsset(String),
    InvalidProtocol(String),
    MissingField(String),
    SameAsset,
    InsufficientBalance,
    EmptyComposite,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub enum Asset {
    Eth,
    Dai,
    Usdc,
    Wbtc,
    Link,
    Sol,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum DexType {
    Uniswap,
    Sushiswap,
    Balancer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum LendingType {
    Aave,
    Compound,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Protocol {
    Dex(DexType),
    Lending(LendingType),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConditionalIntent {
    pub intent: Intent,
    pub condition: Option<Condition>,
    /// Target network for execution (`OTTER_NETWORKS` key). `None` routes to
    /// the default network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
        let decimals = self.decimals() as u32;
        let normalized = amount_str.trim().replace(',', "");

        let (integer_part, decimal_part) = match normalized.split_once('.') {
            Some((int_part, dec_part)) => (int_part, Some(dec_part)),
            None => (normalized.as_str(), None),
        };

        let integer_value = if integer_part.is_empty() {
            0
        } else {
            integer_part.parse::<u128>().ok()?
        };

        let mut amount = integer_value.checked_mul(10u128.pow(decimals))?;

        if let Some(decimal_str) = decimal_part {
            if decimal_str.is_empty() {
                return Some(amount);
            }

            if !decimal_str.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }

            let decimal_len = decimal_str.len() as u32;
            if decimal_len > decimals {
                return None;
            }

            let fractional_value = decimal_str.parse::<u128>().ok()?;
            let scale = decimals - decimal_len;
            let fractional_value = fractional_value.checked_mul(10u128.pow(scale))?;

            amount = amount.checked_add(fractional_value)?;
        }

        Some(amount)
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

    /// Return the ERC-20 token address for this asset on the given chain.
    /// `None` means the native asset (ETH). Hard-coded for Sepolia and Mainnet.
    ///
    /// For testnet overrides, set `OTTER_TOKEN_USDC`, `OTTER_TOKEN_DAI`, etc.
    /// to a 20-byte hex address (with or without `0x`).
    pub fn token_address(&self, chain_id: u64) -> Option<[u8; 20]> {
        if let Some(override_hex) = self.token_address_override() {
            return override_hex;
        }

        let hex = match (self, chain_id) {
            (Asset::Eth, _) => return None,
            // Sepolia
            (Asset::Usdc, 11155111) => "1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
            (Asset::Dai, 11155111) => "3e622317f8C93f7328350cF0B5d5a45cD2E138D8",
            (Asset::Link, 11155111) => "779877A7B0D9E8603169DdbD7836e478b4624789",
            (Asset::Wbtc, 11155111) => "8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063",
            // Mainnet
            (Asset::Usdc, 1) => "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            (Asset::Dai, 1) => "6B175474E89094C44Da98b954EedeAC495271d0F",
            (Asset::Link, 1) => "514910771AF9Ca656af840dff83E8264EcF986CA",
            (Asset::Wbtc, 1) => "2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
            _ => return None,
        };
        let mut bytes = [0u8; 20];
        hex::decode_to_slice(hex, &mut bytes).ok()?;
        Some(bytes)
    }

    fn token_address_override(&self) -> Option<Option<[u8; 20]>> {
        let key = format!("OTTER_TOKEN_{}", self.env_key()?);
        let hex = std::env::var(&key).ok()?;
        let cleaned = hex.trim().strip_prefix("0x").unwrap_or(&hex);
        let mut bytes = [0u8; 20];
        hex::decode_to_slice(cleaned, &mut bytes).ok()?;
        Some(Some(bytes))
    }

    fn env_key(&self) -> Option<&'static str> {
        match self {
            Asset::Eth => None,
            Asset::Usdc => Some("USDC"),
            Asset::Dai => Some("DAI"),
            Asset::Wbtc => Some("WBTC"),
            Asset::Link => Some("LINK"),
            Asset::Sol => Some("SOL"),
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
                Self::ensure_positive_amount(*amount)?;
                if from_asset == to_asset {
                    return Err(ValidationError::SameAsset);
                }
                Ok(())
            }
            Intent::Lend { amount, .. } | Intent::Stake { amount, .. } => {
                Self::ensure_positive_amount(*amount)
            }
            Intent::Borrow {
                amount,
                collateral_amount,
                ..
            } => {
                Self::ensure_positive_amount(*amount)?;
                Self::ensure_positive_amount(*collateral_amount)?;
                Ok(())
            }
            Intent::Composite { intents } => {
                if intents.is_empty() {
                    return Err(ValidationError::EmptyComposite);
                }
                for intent in intents {
                    intent.validate()?;
                }
                Ok(())
            }
        }
    }

    fn ensure_positive_amount(amount: u128) -> Result<(), ValidationError> {
        if amount == 0 {
            Err(ValidationError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Asset, DexType, Intent, LendingType, ValidationError};

    #[test]
    fn parse_amount_rejects_excess_precision() {
        assert!(Asset::Usdc.parse_amount("1.1234567").is_none());
    }

    #[test]
    fn parse_amount_handles_fractional_values() {
        assert_eq!(Asset::Usdc.parse_amount("1.123456"), Some(1_123_456));
        assert_eq!(
            Asset::Eth.parse_amount("2").unwrap(),
            2_000_000_000_000_000_000
        );
    }

    #[test]
    fn swap_requires_positive_amount_and_different_assets() {
        let swap_same_asset = Intent::Swap {
            from_asset: Asset::Eth,
            to_asset: Asset::Eth,
            amount: 1,
            protocol: DexType::Uniswap,
        };
        assert_eq!(swap_same_asset.validate(), Err(ValidationError::SameAsset));
    }

    #[test]
    fn swap_zero_amount_is_rejected() {
        let swap_zero = Intent::Swap {
            from_asset: Asset::Eth,
            to_asset: Asset::Usdc,
            amount: 0,
            protocol: DexType::Uniswap,
        };
        assert!(matches!(
            swap_zero.validate(),
            Err(ValidationError::InvalidAmount(_))
        ));
    }

    #[test]
    fn composite_requires_children() {
        let empty = Intent::Composite { intents: vec![] };
        assert_eq!(empty.validate(), Err(ValidationError::EmptyComposite));
    }

    #[test]
    fn lend_zero_amount_is_rejected() {
        let lend_intent = Intent::Lend {
            asset: Asset::Dai,
            amount: 0,
            protocol: LendingType::Aave,
        };
        assert!(matches!(
            lend_intent.validate(),
            Err(ValidationError::InvalidAmount(_))
        ));
    }

    #[test]
    fn borrow_requires_positive_collateral() {
        let borrow_no_collateral = Intent::Borrow {
            asset: Asset::Usdc,
            amount: 100,
            collateral: Asset::Eth,
            collateral_amount: 0,
            protocol: LendingType::Aave,
        };
        assert!(matches!(
            borrow_no_collateral.validate(),
            Err(ValidationError::InvalidAmount(_))
        ));
    }
}
