use domain::models::{Asset, DexType, Intent, LendingType};
use regex::Regex;

pub struct RegexParser;

impl RegexParser {
    pub fn parse_lend(text: &str) -> Option<Intent> {
        let re =
            Regex::new(r"(?i)lend\s+(?P<amount>[\d,]+)\s+(?P<asset>\w+)\s+on\s+(?P<protocol>\w+)")
                .unwrap();

        if let Some(caps) = re.captures(text) {
            let amount_str = caps.name("amount")?.as_str().replace(",", "");
            let amount: u64 = amount_str.parse().ok()?;
            let asset_str = caps.name("asset")?.as_str().to_lowercase();
            let protocol_str = caps.name("protocol")?.as_str().to_lowercase();
            let asset = match asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return None,
            };
            let protocol = match protocol_str.as_str() {
                "aave" => LendingType::Aave,
                "compound" => LendingType::Compound,
                _ => return None,
            };
            return Some(Intent::Lend {
                asset,
                amount,
                protocol,
            });
        }
        None
    }
    pub fn parse_swap(text: &str) -> Option<Intent> {
        let re = Regex::new(r"(?i)swap\s+(?P<amount>[\d,]+)\s+(?P<from_asset>\w+)\s+for\s+(?P<to_asset>\w+)\s+on\s+(?P<protocol>\w+)")
            .unwrap();

        if let Some(caps) = re.captures(text) {
            let amount_str = caps.name("amount")?.as_str().replace(",", "");
            let amount: u64 = amount_str.parse().ok()?;
            let from_asset_str = caps.name("from_asset")?.as_str().to_lowercase();
            let to_asset_str = caps.name("to_asset")?.as_str().to_lowercase();
            let protocol_str = caps.name("protocol")?.as_str().to_lowercase();
            let from_asset = match from_asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return None,
            };
            let to_asset = match to_asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return None,
            };
            let protocol = match protocol_str.as_str() {
                "uniswap" => DexType::Uniswap,
                "sushiswap" => DexType::Sushiswap,
                "balancer" => DexType::Balancer,
                _ => return None,
            };
            return Some(Intent::Swap {
                from_asset,
                to_asset,
                amount,
                protocol,
            });
        }
        None
    }
    pub fn parse_borrow(text: &str) -> Option<Intent> {
        let re = Regex::new(
            r"(?i)borrow\s+(?P<amount>[\d,]+)\s+(?P<asset>\w+)\s+on\s+(?P<protocol>\w+)",
        )
        .unwrap();

        if let Some(caps) = re.captures(text) {
            let amount_str = caps.name("amount")?.as_str().replace(",", "");
            let amount: u64 = amount_str.parse().ok()?;
            let asset_str = caps.name("asset")?.as_str().to_lowercase();
            let protocol_str = caps.name("protocol")?.as_str().to_lowercase();
            let asset = match asset_str.as_str() {
                "eth" => Asset::Eth,
                "dai" => Asset::Dai,
                "usdc" => Asset::Usdc,
                "wbtc" => Asset::Wbtc,
                "link" => Asset::Link,
                "sol" => Asset::Sol,
                _ => return None,
            };
            let protocol = match protocol_str.as_str() {
                "aave" => LendingType::Aave,
                "compound" => LendingType::Compound,
                _ => return None,
            };
            return Some(Intent::Borrow {
                asset,
                amount,
                protocol,
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lend_basic() {
        let result = RegexParser::parse_lend("Lend 1000 USDC on Aave");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_lend_case_insensitive() {
        let result = RegexParser::parse_lend("lend 1,000 usdc on aave");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_swap_basic() {
        let result = RegexParser::parse_swap("Swap 1 ETH for USDC on Uniswap");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_swap_case_insensitive() {
        let result = RegexParser::parse_swap("swap 1,000 dai for wbtc on sushiswap");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_borrow_basic() {
        let result = RegexParser::parse_borrow("Borrow 1000 USDC on Aave");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_borrow_case_insensitive() {
        let result = RegexParser::parse_borrow("borrow 1,000 usdc on aave");
        assert!(result.is_some());
    }
}
