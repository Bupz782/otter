use crate::models::Intent;

pub struct IntentValidator;

impl IntentValidator {
    /// Validate an Intent against business rules
    /// 
    /// Rules:
    /// - All intents: amount > 0
    /// - Swap: from_asset != to_asset
    /// - All intents: user balance >= amount (checked later with blockchain state)
    pub fn validate(intent: &Intent) -> Result<(), ValidationError> {
        todo!("Will be implemented in Vague 1")
    }
}

#[derive(Debug)]
pub enum ValidationError {
    ZeroAmount,
    SameAsset,
    InsufficientBalance,
}