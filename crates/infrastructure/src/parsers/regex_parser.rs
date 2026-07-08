use super::error::ParseError;
use domain::models::{
    Asset, Comparator, Condition, ConditionalIntent, DexType, Intent, LendingType, Metric,
};
use regex::Regex;
use std::sync::OnceLock;

pub trait IntentParser {
    fn parse(&self, text: &str) -> Result<Intent, ParseError>;

    fn description(&self) -> &'static str;
}

static LEND_REGEX: OnceLock<Regex> = OnceLock::new();
static SWAP_REGEX: OnceLock<Regex> = OnceLock::new();
static BORROW_REGEX: OnceLock<Regex> = OnceLock::new();
static STAKE_REGEX: OnceLock<Regex> = OnceLock::new();
static CONDITION_REGEX: OnceLock<Regex> = OnceLock::new();
static CONDITIONAL_SPLIT_REGEX: OnceLock<Regex> = OnceLock::new();

mod parsers {
    use super::*;

    pub fn parse_asset(s: &str) -> Result<Asset, ParseError> {
        match s.to_lowercase().as_str() {
            "eth" => Ok(Asset::Eth),
            "dai" => Ok(Asset::Dai),
            "usdc" => Ok(Asset::Usdc),
            "wbtc" => Ok(Asset::Wbtc),
            "link" => Ok(Asset::Link),
            "sol" => Ok(Asset::Sol),
            _ => Err(ParseError::UnknownAsset(s.to_string())),
        }
    }

    pub fn parse_dex_protocol(s: &str) -> Result<DexType, ParseError> {
        match s.to_lowercase().as_str() {
            "uniswap" => Ok(DexType::Uniswap),
            "sushiswap" => Ok(DexType::Sushiswap),
            "balancer" => Ok(DexType::Balancer),
            _ => Err(ParseError::UnknownProtocol(s.to_string())),
        }
    }

    pub fn parse_lending_protocol(s: &str) -> Result<LendingType, ParseError> {
        match s.to_lowercase().as_str() {
            "aave" => Ok(LendingType::Aave),
            "compound" => Ok(LendingType::Compound),
            _ => Err(ParseError::UnknownProtocol(s.to_string())),
        }
    }

    pub fn parse_amount(caps: &regex::Captures, asset: &Asset) -> Result<u128, ParseError> {
        let amount_str = caps
            .name("amount")
            .ok_or(ParseError::InvalidFormat("Missing amount".to_string()))?
            .as_str()
            .replace(",", "");

        asset
            .parse_amount(&amount_str)
            .ok_or(ParseError::InvalidAmount(amount_str))
    }

    pub fn parse_comparator(s: &str) -> Result<Comparator, ParseError> {
        match s {
            ">" => Ok(Comparator::GreaterThan),
            "<" => Ok(Comparator::LessThan),
            "=" => Ok(Comparator::EqualTo),
            "<=" => Ok(Comparator::LessThanOrEqualTo),
            ">=" => Ok(Comparator::GreaterThanOrEqualTo),
            _ => Err(ParseError::InvalidFormat("Unknown comparator".to_string())),
        }
    }

    pub fn parse_metric(s: &str) -> Result<Metric, ParseError> {
        match s.to_lowercase().as_str() {
            "price" => Ok(Metric::Price),
            "volume" => Ok(Metric::Volume),
            "gascost" | "gas_cost" | "gas" => Ok(Metric::GasCost),
            "yield" => Ok(Metric::Yield),
            _ => Err(ParseError::UnknownMetric(s.to_string())),
        }
    }
}

pub struct LendParser;

impl IntentParser for LendParser {
    fn description(&self) -> &'static str {
        "Parses lend intents: 'lend <amount> <asset> on <protocol>'"
    }

    fn parse(&self, text: &str) -> Result<Intent, ParseError> {
        let re = LEND_REGEX.get_or_init(|| {
            Regex::new(r"(?i)lend\s+(?P<amount>[\d,\.]+)\s+(?P<asset>\w+)\s+on\s+(?P<protocol>\w+)")
                .expect("Invalid LEND_REGEX pattern")
        });

        let caps = re.captures(text).ok_or(ParseError::InvalidFormat(
            "Could not parse lend intent".to_string(),
        ))?;

        let asset_str = caps
            .name("asset")
            .ok_or(ParseError::InvalidFormat("Missing asset".to_string()))?
            .as_str();
        let asset = parsers::parse_asset(asset_str)?;

        let amount = parsers::parse_amount(&caps, &asset)?;

        let protocol_str = caps
            .name("protocol")
            .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
            .as_str();
        let protocol = parsers::parse_lending_protocol(protocol_str)?;

        Ok(Intent::Lend {
            asset,
            amount,
            protocol,
        })
    }
}

pub struct SwapParser;

impl IntentParser for SwapParser {
    fn description(&self) -> &'static str {
        "Parses swap intents: 'swap <amount> <from_asset> for <to_asset> on <protocol>'"
    }

    fn parse(&self, text: &str) -> Result<Intent, ParseError> {
        let re = SWAP_REGEX.get_or_init(|| {
            Regex::new(r"(?i)swap\s+(?P<amount>[\d,\.]+)\s+(?P<from_asset>\w+)\s+for\s+(?P<to_asset>\w+)\s+on\s+(?P<protocol>\w+)")
                .expect("Invalid SWAP_REGEX pattern")
        });

        let caps = re.captures(text).ok_or(ParseError::InvalidFormat(
            "Could not parse swap intent".to_string(),
        ))?;

        let from_asset_str = caps
            .name("from_asset")
            .ok_or(ParseError::InvalidFormat("Missing from_asset".to_string()))?
            .as_str();
        let from_asset = parsers::parse_asset(from_asset_str)?;

        let amount = parsers::parse_amount(&caps, &from_asset)?;

        let to_asset_str = caps
            .name("to_asset")
            .ok_or(ParseError::InvalidFormat("Missing to_asset".to_string()))?
            .as_str();
        let to_asset = parsers::parse_asset(to_asset_str)?;

        let protocol_str = caps
            .name("protocol")
            .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
            .as_str();
        let protocol = parsers::parse_dex_protocol(protocol_str)?;

        Ok(Intent::Swap {
            from_asset,
            to_asset,
            amount,
            protocol,
        })
    }
}

pub struct BorrowParser;

impl IntentParser for BorrowParser {
    fn description(&self) -> &'static str {
        "Parses borrow intents: 'borrow <amount> <asset> with <collateral_amount> <collateral> on <protocol>'"
    }

    fn parse(&self, text: &str) -> Result<Intent, ParseError> {
        let re = BORROW_REGEX.get_or_init(|| {
            Regex::new(
                r"(?i)borrow\s+(?P<amount>[\d,\.]+)\s+(?P<asset>\w+)\s+with\s+(?P<collateral_amount>[\d,\.]+)\s+(?P<collateral>\w+)\s+on\s+(?P<protocol>\w+)",
            )
            .expect("Invalid BORROW_REGEX pattern")
        });

        let caps = re.captures(text).ok_or(ParseError::InvalidFormat(
            "Could not parse borrow intent. Format: borrow <amount> <asset> with <collateral_amount> <collateral> on <protocol>".to_string(),
        ))?;

        let asset_str = caps
            .name("asset")
            .ok_or(ParseError::InvalidFormat("Missing asset".to_string()))?
            .as_str();
        let asset = parsers::parse_asset(asset_str)?;

        let amount = parsers::parse_amount(&caps, &asset)?;

        let collateral_str = caps
            .name("collateral")
            .ok_or(ParseError::InvalidFormat("Missing collateral".to_string()))?
            .as_str();
        let collateral = parsers::parse_asset(collateral_str)?;

        let collateral_amount_str = caps
            .name("collateral_amount")
            .ok_or(ParseError::InvalidFormat(
                "Missing collateral amount".to_string(),
            ))?
            .as_str()
            .replace(",", "");
        let collateral_amount = collateral
            .parse_amount(&collateral_amount_str)
            .ok_or(ParseError::InvalidAmount(collateral_amount_str))?;

        let protocol_str = caps
            .name("protocol")
            .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
            .as_str();
        let protocol = parsers::parse_lending_protocol(protocol_str)?;

        Ok(Intent::Borrow {
            asset,
            amount,
            collateral,
            collateral_amount,
            protocol,
        })
    }
}

pub struct StakeParser;

impl IntentParser for StakeParser {
    fn description(&self) -> &'static str {
        "Parses stake intents: 'stake <amount> <asset> on <protocol>'"
    }

    fn parse(&self, text: &str) -> Result<Intent, ParseError> {
        let re = STAKE_REGEX.get_or_init(|| {
            Regex::new(
                r"(?i)stake\s+(?P<amount>[\d,\.]+)\s+(?P<asset>\w+)\s+on\s+(?P<protocol>\w+)",
            )
            .expect("Invalid STAKE_REGEX pattern")
        });

        let caps = re.captures(text).ok_or(ParseError::InvalidFormat(
            "Could not parse stake intent".to_string(),
        ))?;

        let asset_str = caps
            .name("asset")
            .ok_or(ParseError::InvalidFormat("Missing asset".to_string()))?
            .as_str();
        let asset = parsers::parse_asset(asset_str)?;

        let amount = parsers::parse_amount(&caps, &asset)?;

        let protocol_str = caps
            .name("protocol")
            .ok_or(ParseError::InvalidFormat("Missing protocol".to_string()))?
            .as_str();
        let protocol = parsers::parse_lending_protocol(protocol_str)?;

        Ok(Intent::Stake {
            asset,
            amount,
            protocol,
        })
    }
}

/// Parser for conditions
pub struct ConditionParser;

impl ConditionParser {
    pub fn parse(&self, text: &str) -> Result<Condition, ParseError> {
        let re = CONDITION_REGEX.get_or_init(|| {
            Regex::new(r"(?i)if\s+(?P<metric>\w+)\s+(?P<comparator>[><=]+)\s+(?P<value>[\d,_]+)")
                .expect("Invalid CONDITION_REGEX pattern")
        });

        let caps = re.captures(text).ok_or(ParseError::InvalidFormat(
            "Could not parse condition".to_string(),
        ))?;

        let value_str = caps
            .name("value")
            .ok_or(ParseError::InvalidFormat("Missing value".to_string()))?
            .as_str()
            .replace(",", "")
            .replace("_", "");
        let value: u128 = value_str
            .parse()
            .map_err(|_| ParseError::InvalidAmount(value_str.clone()))?;

        let metric_str = caps
            .name("metric")
            .ok_or(ParseError::InvalidFormat("Missing metric".to_string()))?
            .as_str();
        let metric = parsers::parse_metric(metric_str)?;

        let comparator_str = caps
            .name("comparator")
            .ok_or(ParseError::InvalidFormat("Missing comparator".to_string()))?
            .as_str();
        let comparator = parsers::parse_comparator(comparator_str)?;

        Ok(Condition::Comparison {
            metric,
            comparator,
            value,
        })
    }
}

/// Main regex parser that tries multiple parsers in sequence
pub struct RegexParser {
    parsers: Vec<Box<dyn IntentParser + Send + Sync>>,
    condition_parser: ConditionParser,
}

impl Default for RegexParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RegexParser {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl RegexParser {
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(LendParser),
                Box::new(BorrowParser),
                Box::new(StakeParser),
                Box::new(SwapParser),
            ],
            condition_parser: ConditionParser,
        }
    }

    /// Parse an intent using all registered parsers
    pub fn parse_intent(&self, text: &str) -> Result<Intent, ParseError> {
        for parser in &self.parsers {
            if let Ok(intent) = parser.parse(text) {
                return Ok(intent);
            }
        }
        Err(ParseError::InvalidFormat(
            "Could not parse intent with any parser".to_string(),
        ))
    }

    /// Parse a condition
    pub fn parse_condition(&self, text: &str) -> Result<Condition, ParseError> {
        self.condition_parser.parse(text)
    }

    /// Parse a conditional intent (intent + optional condition)
    pub fn parse_conditional_intent(&self, text: &str) -> Result<ConditionalIntent, ParseError> {
        let splitter = CONDITIONAL_SPLIT_REGEX
            .get_or_init(|| Regex::new(r"(?i)\bif\b").expect("Invalid conditional split regex"));

        let text = text.trim();
        let (intent_part, condition_slice) = if let Some(mat) = splitter.find(text) {
            let intent_part = text[..mat.start()].trim();
            let condition_part = text[mat.end()..].trim();
            (intent_part, Some(condition_part))
        } else {
            (text, None)
        };

        if intent_part.is_empty() {
            return Err(ParseError::InvalidFormat("Missing intent part".to_string()));
        }

        let intent = self.parse_intent(intent_part)?;

        let condition = if let Some(cond_text) = condition_slice {
            Some(self.parse_condition(&format!("if {}", cond_text))?)
        } else {
            None
        };

        Ok(ConditionalIntent { intent, condition })
    }

    /// Get descriptions of all registered parsers
    pub fn parser_descriptions(&self) -> Vec<&'static str> {
        self.parsers.iter().map(|p| p.description()).collect()
    }
}

impl domain::ports::intent_parser_port::IntentParserPort for RegexParser {
    fn parse(
        &self,
        text: &str,
    ) -> Result<ConditionalIntent, domain::ports::intent_parser_port::IntentParserError> {
        self.parse_conditional_intent(text).map_err(|e| {
            domain::ports::intent_parser_port::IntentParserError::ParsingFailed(e.to_string())
        })
    }
}

// Backward-compatible API
impl RegexParser {
    pub fn parse_lend(text: &str) -> Result<Intent, ParseError> {
        LendParser.parse(text)
    }

    pub fn parse_swap(text: &str) -> Result<Intent, ParseError> {
        SwapParser.parse(text)
    }

    pub fn parse_borrow(text: &str) -> Result<Intent, ParseError> {
        BorrowParser.parse(text)
    }

    pub fn parse_stake(text: &str) -> Result<Intent, ParseError> {
        StakeParser.parse(text)
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
        let result = RegexParser::parse_borrow("Borrow 1000 USDC with 1.5 ETH on Aave");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_borrow_case_insensitive() {
        let result = RegexParser::parse_borrow("borrow 1,000 usdc with 1.5 eth on aave");
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
        let parser = RegexParser::new();
        let result = parser.parse_condition("if yield > 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_condition_gas() {
        let parser = RegexParser::new();
        let result = parser.parse_condition("if gas <= 50");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_conditional_lend() {
        let parser = RegexParser::new();
        let result = parser.parse_conditional_intent("Lend 1000 USDC on Aave if yield > 5");
        assert!(result.is_ok());
        let ci = result.unwrap();
        assert!(ci.condition.is_some());
    }

    #[test]
    fn test_parse_unconditional_lend() {
        let parser = RegexParser::new();
        let result = parser.parse_conditional_intent("Lend 1000 USDC on Aave");
        assert!(result.is_ok());
        let ci = result.unwrap();
        assert!(ci.condition.is_none());
    }

    #[test]
    fn test_parse_conditional_handles_uppercase_if() {
        let parser = RegexParser::new();
        let result =
            parser.parse_conditional_intent("Swap 1 ETH for USDC on Uniswap IF price > 2000");
        assert!(result.is_ok());
        let ci = result.unwrap();
        assert!(ci.condition.is_some());
    }

    #[test]
    fn test_parse_conditional_without_space_before_if_keyword() {
        let parser = RegexParser::new();
        let result =
            parser.parse_conditional_intent("Swap 1 ETH for USDC on Uniswap.if price > 2000");
        assert!(result.is_ok());
        let ci = result.unwrap();
        assert!(ci.condition.is_some());
    }

    #[test]
    fn test_parse_condition_underscore_and_comma_value() {
        let parser = RegexParser::new();
        let result = parser.parse_condition("if price > 4_000_000");
        assert!(result.is_ok());
        if let Ok(Condition::Comparison { value, .. }) = result {
            assert_eq!(value, 4_000_000);
        } else {
            panic!("Expected comparison condition");
        }

        let result = parser.parse_condition("if price >= 2,500,000");
        assert!(result.is_ok());
        if let Ok(Condition::Comparison { value, .. }) = result {
            assert_eq!(value, 2_500_000);
        } else {
            panic!("Expected comparison condition");
        }
    }

    #[test]
    fn test_trait_parser_lend() {
        let parser = LendParser;
        let result = parser.parse("lend 500 DAI on Compound");
        assert!(result.is_ok());

        if let Ok(Intent::Lend {
            asset,
            amount,
            protocol,
        }) = result
        {
            assert_eq!(asset, Asset::Dai);
            assert_eq!(protocol, LendingType::Compound);
            assert!(amount > 0);
        } else {
            panic!("Expected Lend intent");
        }
    }

    #[test]
    fn test_trait_parser_swap() {
        let parser = SwapParser;
        let result = parser.parse("swap 1.5 ETH for USDC on Uniswap");
        assert!(result.is_ok());

        if let Ok(Intent::Swap {
            from_asset,
            to_asset,
            protocol,
            ..
        }) = result
        {
            assert_eq!(from_asset, Asset::Eth);
            assert_eq!(to_asset, Asset::Usdc);
            assert_eq!(protocol, DexType::Uniswap);
        } else {
            panic!("Expected Swap intent");
        }
    }

    #[test]
    fn test_regex_compiled_once() {
        let result1 = RegexParser::parse_lend("lend 100 USDC on Aave");
        let result2 = RegexParser::parse_lend("lend 200 DAI on Compound");
        let result3 = RegexParser::parse_lend("lend 300 WBTC on Aave");

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
    }

    #[test]
    fn test_parser_descriptions() {
        let parser = RegexParser::new();
        let descriptions = parser.parser_descriptions();
        assert_eq!(descriptions.len(), 4);

        for desc in descriptions {
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_parse_intent_auto_detect() {
        let parser = RegexParser::new();

        let result = parser.parse_intent("lend 1000 USDC on Aave");
        assert!(matches!(result, Ok(Intent::Lend { .. })));

        let result = parser.parse_intent("swap 1 ETH for DAI on Uniswap");
        assert!(matches!(result, Ok(Intent::Swap { .. })));

        let result = parser.parse_intent("borrow 500 DAI with 1 ETH on Compound");
        assert!(matches!(result, Ok(Intent::Borrow { .. })));

        let result = parser.parse_intent("stake 1000 USDC on Aave");
        assert!(matches!(result, Ok(Intent::Stake { .. })));
    }
}
