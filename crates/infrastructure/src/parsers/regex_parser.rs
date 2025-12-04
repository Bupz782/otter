use regex::Regex;
use domain::models::{Intent, Asset, LendingType};

pub struct RegexParser;

impl RegexParser {
    pub fn parse_lend(text: &str) -> Option<Intent> {
        let re = Regex::new(r"(?i)lend\s+(?P<amount>[\d,]+)\s+(?P<asset>\w+)\s+on\s+(?P<protocol>\w+)"
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
            return Some(Intent::Lend {
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
}