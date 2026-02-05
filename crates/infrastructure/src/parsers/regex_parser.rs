use super::error::ParseError;
use domain::models::{
    Asset, Comparator, Condition, ConditionalIntent, DexType, Intent, LendingType, Metric,
};
use regex::Regex;

pub struct RegexParser;

impl RegexParser {
    pub fn parse_lend(text: &str) -> Result<Intent, ParseError> {
        let re = Regex::new(
            r"(?i)lend\s+(?P<amount>[\d,\.]+)\s+(?P<asset>\w+)\s+on\s+(?P<protocol>\w+)",
        )
        .unwrap();

        if let Some(caps) = re.captures(text) {
            let amount_str = caps
                .name("amount")
                .ok_or(ParseError::InvalidFormat("Missing amount".to_string()))?
                .as_str()
                .replace(",", "");
            let asset_str = caps
                .name("asset")
                .ok_or(ParseError::InvalidFormat("Missing asset".to_string()))?
                .as_str()
                .to_lowercase();
            let protocol_str = caps
                .name("protocol")
                .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
                .as_str()
                .to_lowercase();
            let asset = match asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return Err(ParseError::UnknownAsset(asset_str)),
            };
            let amount: u128 = asset
                .parse_amount(&amount_str)
                .ok_or(ParseError::InvalidAmount(amount_str.clone()))?;
            let protocol = match protocol_str.as_str() {
                "aave" => LendingType::Aave,
                "compound" => LendingType::Compound,
                _ => return Err(ParseError::UnknownProtocol(protocol_str)),
            };
            return Ok(Intent::Lend {
                asset,
                amount,
                protocol,
            });
        }
        Err(ParseError::InvalidFormat(
            "Could not parse lend intent".to_string(),
        ))
    }
    pub fn parse_swap(text: &str) -> Result<Intent, ParseError> {
        let re = Regex::new(r"(?i)swap\s+(?P<amount>[\d,\.]+)\s+(?P<from_asset>\w+)\s+for\s+(?P<to_asset>\w+)\s+on\s+(?P<protocol>\w+)")
            .unwrap();

        if let Some(caps) = re.captures(text) {
            let amount_str = caps
                .name("amount")
                .ok_or(ParseError::InvalidFormat("Missing amount".to_string()))?
                .as_str()
                .replace(",", "");
            let from_asset_str = caps
                .name("from_asset")
                .ok_or(ParseError::InvalidFormat("Missing from_asset".to_string()))?
                .as_str()
                .to_lowercase();
            let to_asset_str = caps
                .name("to_asset")
                .ok_or(ParseError::InvalidFormat("Missing to_asset".to_string()))?
                .as_str()
                .to_lowercase();
            let protocol_str = caps
                .name("protocol")
                .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
                .as_str()
                .to_lowercase();
            let from_asset = match from_asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return Err(ParseError::UnknownAsset(from_asset_str)),
            };
            let amount: u128 = from_asset
                .parse_amount(&amount_str)
                .ok_or(ParseError::InvalidAmount(amount_str.clone()))?;
            let to_asset = match to_asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return Err(ParseError::UnknownAsset(to_asset_str)),
            };
            let protocol = match protocol_str.as_str() {
                "uniswap" => DexType::Uniswap,
                "sushiswap" => DexType::Sushiswap,
                "balancer" => DexType::Balancer,
                _ => return Err(ParseError::UnknownProtocol(protocol_str)),
            };
            return Ok(Intent::Swap {
                from_asset,
                to_asset,
                amount,
                protocol,
            });
        }
        Err(ParseError::InvalidFormat(
            "Could not parse swap intent".to_string(),
        ))
    }
    pub fn parse_borrow(text: &str) -> Result<Intent, ParseError> {
        // Format: "borrow 100 USDC with 1.5 ETH on aave"
        let re = Regex::new(
            r"(?i)borrow\s+(?P<amount>[\d,\.]+)\s+(?P<asset>\w+)\s+with\s+(?P<collateral_amount>[\d,\.]+)\s+(?P<collateral>\w+)\s+on\s+(?P<protocol>\w+)",
        )
        .unwrap();

        if let Some(caps) = re.captures(text) {
            let amount_str = caps
                .name("amount")
                .ok_or(ParseError::InvalidFormat("Missing amount".to_string()))?
                .as_str()
                .replace(",", "");
            let asset_str = caps
                .name("asset")
                .ok_or(ParseError::InvalidFormat("Missing asset".to_string()))?
                .as_str()
                .to_lowercase();
            let collateral_amount_str = caps
                .name("collateral_amount")
                .ok_or(ParseError::InvalidFormat("Missing collateral amount".to_string()))?
                .as_str()
                .replace(",", "");
            let collateral_str = caps
                .name("collateral")
                .ok_or(ParseError::InvalidFormat("Missing collateral".to_string()))?
                .as_str()
                .to_lowercase();
            let protocol_str = caps
                .name("protocol")
                .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
                .as_str()
                .to_lowercase();

            let parse_asset = |s: &str| -> Result<Asset, ParseError> {
                match s {
                    "eth" => Ok(Asset::Eth),
                    "dai" => Ok(Asset::Dai),
                    "usdc" => Ok(Asset::Usdc),
                    "wbtc" => Ok(Asset::Wbtc),
                    "link" => Ok(Asset::Link),
                    "sol" => Ok(Asset::Sol),
                    _ => Err(ParseError::UnknownAsset(s.to_string())),
                }
            };

            let asset = parse_asset(&asset_str)?;
            let collateral = parse_asset(&collateral_str)?;

            let amount: u128 = asset
                .parse_amount(&amount_str)
                .ok_or(ParseError::InvalidAmount(amount_str.clone()))?;
            let collateral_amount: u128 = collateral
                .parse_amount(&collateral_amount_str)
                .ok_or(ParseError::InvalidAmount(collateral_amount_str.clone()))?;

            let protocol = match protocol_str.as_str() {
                "aave" => LendingType::Aave,
                "compound" => LendingType::Compound,
                _ => return Err(ParseError::UnknownProtocol(protocol_str)),
            };

            return Ok(Intent::Borrow {
                asset,
                amount,
                collateral,
                collateral_amount,
                protocol,
            });
        }
        Err(ParseError::InvalidFormat(
            "Could not parse borrow intent. Format: borrow <amount> <asset> with <collateral_amount> <collateral> on <protocol>".to_string(),
        ))
    }
    pub fn parse_stake(text: &str) -> Result<Intent, ParseError> {
        let re = Regex::new(
            r"(?i)stake\s+(?P<amount>[\d,\.]+)\s+(?P<asset>\w+)\s+on\s+(?P<protocol>\w+)",
        )
        .unwrap();

        if let Some(caps) = re.captures(text) {
            let amount_str = caps
                .name("amount")
                .ok_or(ParseError::InvalidFormat("Missing amount".to_string()))?
                .as_str()
                .replace(",", "");
            let asset_str = caps
                .name("asset")
                .ok_or(ParseError::InvalidFormat("Missing asset".to_string()))?
                .as_str()
                .to_lowercase();
            let protocol_str = caps
                .name("protocol")
                .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
                .as_str()
                .to_lowercase();
            let asset = match asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return Err(ParseError::UnknownAsset(asset_str)),
            };
            let amount: u128 = asset
                .parse_amount(&amount_str)
                .ok_or(ParseError::InvalidAmount(amount_str.clone()))?;
            let protocol = match protocol_str.as_str() {
                "aave" => LendingType::Aave,
                "compound" => LendingType::Compound,
                _ => return Err(ParseError::UnknownProtocol(protocol_str)),
            };
            return Ok(Intent::Stake {
                asset,
                amount,
                protocol,
            });
        }
        Err(ParseError::InvalidFormat(
            "Could not parse stake intent".to_string(),
        ))
    }
    pub fn parse_condition(text: &str) -> Result<Condition, ParseError> {
        let re =
            Regex::new(r"(?i)if\s+(?P<metric>\w+)\s+(?P<comparator>[><=]+)\s+(?P<value>[\d,]+)")
                .unwrap();

        if let Some(caps) = re.captures(text) {
            let value_str = caps
                .name("value")
                .ok_or(ParseError::InvalidFormat("Missing value".to_string()))?
                .as_str()
                .replace(",", "");
            let value: u128 = value_str
                .parse()
                .map_err(|_| ParseError::InvalidAmount(value_str.clone()))?;
            let metric_str = caps
                .name("metric")
                .ok_or(ParseError::InvalidFormat("Missing metric".to_string()))?
                .as_str()
                .to_lowercase();
            let comparator_str = caps
                .name("comparator")
                .ok_or(ParseError::InvalidFormat("Missing comparator".to_string()))?
                .as_str();
            let metric = match metric_str.as_str() {
                "price" => Metric::Price,
                "volume" => Metric::Volume,
                "gascost" | "gas_cost" | "gas" => Metric::GasCost,
                "yield" => Metric::Yield,
                _ => return Err(ParseError::UnknownMetric(metric_str)),
            };
            let comparator = match comparator_str {
                ">" => Comparator::GreaterThan,
                "<" => Comparator::LessThan,
                "=" => Comparator::EqualTo,
                "<=" => Comparator::LessThanOrEqualTo,
                ">=" => Comparator::GreaterThanOrEqualTo,
                _ => return Err(ParseError::InvalidFormat("Unknown comparator".to_string())),
            };
            return Ok(Condition::Comparison {
                metric,
                comparator,
                value,
            });
        }
        Err(ParseError::InvalidFormat(
            "Could not parse condition".to_string(),
        ))
    }
    pub fn parse_conditional_intent(text: &str) -> Result<ConditionalIntent, ParseError> {
        let mut parts = text.splitn(2, " if ");
        let intent_part = parts
            .next()
            .ok_or(ParseError::InvalidFormat("Missing intent part".to_string()))?;
        let condition_part = parts.next();
        let intent = Self::parse_lend(intent_part)
            .or_else(|_| Self::parse_borrow(intent_part))
            .or_else(|_| Self::parse_stake(intent_part))
            .or_else(|_| Self::parse_swap(intent_part))?;
        let condition = if let Some(cond_text) = condition_part {
            Some(Self::parse_condition(&format!("if {}", cond_text))?)
        } else {
            None
        };
        Ok(ConditionalIntent { intent, condition })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lend_basic() {
        let result = RegexParser::parse_lend("Lend 1000 USDC on Aave");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lend_case_insensitive() {
        let result = RegexParser::parse_lend("lend 1,000 usdc on aave");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_swap_basic() {
        let result = RegexParser::parse_swap("Swap 1 ETH for USDC on Uniswap");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_swap_case_insensitive() {
        let result = RegexParser::parse_swap("swap 1,000 dai for wbtc on sushiswap");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_borrow_basic() {
        let result = RegexParser::parse_borrow("Borrow 1000 USDC on Aave");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_borrow_case_insensitive() {
        let result = RegexParser::parse_borrow("borrow 1,000 usdc on aave");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_stake_basic() {
        let result = RegexParser::parse_stake("Stake 1000 USDC on Aave");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_stake_case_insensitive() {
        let result = RegexParser::parse_stake("stake 1,000 usdc on aave");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_condition_yield() {
        let result = RegexParser::parse_condition("if yield > 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_condition_gas() {
        let result = RegexParser::parse_condition("if gas <= 50");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_conditional_lend() {
        let result = RegexParser::parse_conditional_intent("Lend 1000 USDC on Aave if yield > 5");
        assert!(result.is_ok());
        let ci = result.unwrap();
        assert!(ci.condition.is_some());
    }

    #[test]
    fn test_parse_unconditional_lend() {
        let result = RegexParser::parse_conditional_intent("Lend 1000 USDC on Aave");
        assert!(result.is_ok());
        let ci = result.unwrap();
        assert!(ci.condition.is_none());
    }
}
