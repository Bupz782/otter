use crate::models::intent::ConditionalIntent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyValidationError {
    EmptyTitle,
    EmptyDescription,
    EmptyRawText,
    InvalidRiskProfile(String),
    InvalidIntent(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Strategy {
    pub id: String,
    pub title: String,
    pub description: String,
    pub raw_text: String,
    pub intent: ConditionalIntent,
    pub creator_address: Option<String>,
    pub agent_id: String,
    pub risk_profile: String,
    pub copies: u64,
    pub total_volume: u64,
    pub apy: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Strategy {
    pub fn validate(&self) -> Result<(), StrategyValidationError> {
        if self.title.trim().is_empty() {
            return Err(StrategyValidationError::EmptyTitle);
        }
        if self.description.trim().is_empty() {
            return Err(StrategyValidationError::EmptyDescription);
        }
        if self.raw_text.trim().is_empty() {
            return Err(StrategyValidationError::EmptyRawText);
        }
        if !matches!(
            self.risk_profile.as_str(),
            "Conservative" | "Balanced" | "Advanced"
        ) {
            return Err(StrategyValidationError::InvalidRiskProfile(
                self.risk_profile.clone(),
            ));
        }
        self.intent
            .intent
            .validate()
            .map_err(|e| StrategyValidationError::InvalidIntent(format!("{:?}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::condition::{Comparator, Condition, Metric};
    use crate::models::intent::{Asset, Intent, LendingType};

    fn sample_strategy() -> Strategy {
        Strategy {
            id: "s-1".to_string(),
            title: "Steady USDC".to_string(),
            description: "Lend USDC when APY > 3%".to_string(),
            raw_text: "Lend 1000 USDC on Aave if yield > 3%".to_string(),
            intent: ConditionalIntent {
                intent: Intent::Lend {
                    asset: Asset::Usdc,
                    amount: 1_000_000_000,
                    protocol: LendingType::Aave,
                },
                condition: Some(Condition::Comparison {
                    metric: Metric::Yield,
                    comparator: Comparator::GreaterThan,
                    value: 3_00,
                }),
            },
            creator_address: None,
            agent_id: "agent-1".to_string(),
            risk_profile: "Conservative".to_string(),
            copies: 0,
            total_volume: 0,
            apy: 0.0,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn strategy_validates_basic_fields() {
        let strategy = sample_strategy();
        assert!(strategy.validate().is_ok());
    }

    #[test]
    fn strategy_rejects_bad_risk_profile() {
        let mut strategy = sample_strategy();
        strategy.risk_profile = "Wild".to_string();
        assert!(matches!(
            strategy.validate(),
            Err(StrategyValidationError::InvalidRiskProfile(_))
        ));
    }

    #[test]
    fn strategy_rejects_empty_title() {
        let mut strategy = sample_strategy();
        strategy.title = "   ".to_string();
        assert_eq!(
            strategy.validate(),
            Err(StrategyValidationError::EmptyTitle)
        );
    }

    #[test]
    fn strategy_rejects_empty_description() {
        let mut strategy = sample_strategy();
        strategy.description = "".to_string();
        assert_eq!(
            strategy.validate(),
            Err(StrategyValidationError::EmptyDescription)
        );
    }

    #[test]
    fn strategy_rejects_empty_raw_text() {
        let mut strategy = sample_strategy();
        strategy.raw_text = "\t\n".to_string();
        assert_eq!(
            strategy.validate(),
            Err(StrategyValidationError::EmptyRawText)
        );
    }

    #[test]
    fn strategy_rejects_invalid_intent() {
        let mut strategy = sample_strategy();
        strategy.intent = ConditionalIntent {
            intent: Intent::Lend {
                asset: Asset::Usdc,
                amount: 0,
                protocol: LendingType::Aave,
            },
            condition: None,
        };
        assert!(matches!(
            strategy.validate(),
            Err(StrategyValidationError::InvalidIntent(_))
        ));
    }
}
