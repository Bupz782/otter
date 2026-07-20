use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use siwe::Message;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors that can occur during authentication.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid SIWE message: {0}")]
    InvalidMessage(String),
    #[error("signature verification failed: {0}")]
    VerificationFailed(String),
    #[error("challenge expired or not found")]
    ChallengeNotFound,
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Authenticated user extracted from a valid JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

struct Challenge {
    message: String,
    expiry: i64,
}

/// Service handling Sign-In with Ethereum (EIP-4361) challenges and JWT issuance.
pub struct AuthService {
    jwt_secret: String,
    /// In-memory challenge store: nonce -> Challenge.
    challenges: Mutex<HashMap<String, Challenge>>,
    token_ttl_hours: i64,
}

impl AuthService {
    /// Create a new auth service. If `jwt_secret` is empty, a random one is generated
    /// (fine for dev, NOT for production).
    pub fn new(jwt_secret: String, token_ttl_hours: i64) -> Self {
        let jwt_secret = if jwt_secret.is_empty() {
            // Generate a random 32-byte secret for dev.
            let bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
            hex::encode(bytes)
        } else {
            jwt_secret
        };
        Self {
            jwt_secret,
            challenges: Mutex::new(HashMap::new()),
            token_ttl_hours,
        }
    }

    /// Generate a new SIWE challenge message for the given Ethereum address.
    /// Returns the message string to be signed by the user.
    pub fn generate_challenge(&self, address: &str) -> Result<String, AuthError> {
        let nonce = hex::encode((0..16).map(|_| rand::random::<u8>()).collect::<Vec<_>>());
        let now = Utc::now();
        let expiration = now + Duration::minutes(5);

        let message = format!(
            "{} wants you to sign in with your Ethereum account:\n{}\n\n\
             Sign in to Otter agent\n\n\
             URI: https://otter.local\n\
             Version: 1\n\
             Chain ID: 1\n\
             Nonce: {}\n\
             Issued At: {}\n\
             Expiration Time: {}",
            "otter.local",
            address,
            nonce,
            now.to_rfc3339(),
            expiration.to_rfc3339()
        );

        let challenge = Challenge {
            message: message.clone(),
            expiry: expiration.timestamp(),
        };

        self.challenges
            .lock()
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .insert(nonce, challenge);

        Ok(message)
    }

    /// Verify a signed SIWE message and return a JWT on success.
    pub fn verify_signature(
        &self,
        message: &str,
        signature_hex: &str,
    ) -> Result<String, AuthError> {
        let msg: Message = message
            .parse()
            .map_err(|e| AuthError::InvalidMessage(format!("{:?}", e)))?;

        // Verify the challenge exists and is not expired.
        {
            let store = self
                .challenges
                .lock()
                .map_err(|e| AuthError::Internal(e.to_string()))?;
            let challenge = store.get(&msg.nonce).ok_or(AuthError::ChallengeNotFound)?;
            if challenge.message != message {
                return Err(AuthError::ChallengeNotFound);
            }
            if now_secs() > challenge.expiry {
                return Err(AuthError::ChallengeNotFound);
            }
        }

        // Parse signature.
        let sig_bytes = hex::decode(signature_hex.trim_start_matches("0x"))
            .map_err(|e| AuthError::VerificationFailed(e.to_string()))?;
        let sig: &[u8; 65] = sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| AuthError::VerificationFailed("signature must be 65 bytes".to_string()))?;

        // Enforce the message's own time constraints (not-before / expiration).
        if !msg.valid_now() {
            return Err(AuthError::VerificationFailed(
                "SIWE message expired or not yet valid".to_string(),
            ));
        }

        // Verify the EIP-191 personal_sign signature locally (no provider
        // needed). This is synchronous so it is safe to call from async
        // handlers.
        msg.verify_eip191(sig)
            .map_err(|e| AuthError::VerificationFailed(format!("{:?}", e)))?;

        // Clean up used challenge.
        self.challenges
            .lock()
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .remove(&msg.nonce);

        let address = siwe::eip55(&msg.address);
        self.issue_token(&address)
    }

    /// Validate a JWT and return the authenticated user address.
    pub fn validate_token(&self, token: &str) -> Result<AuthUser, AuthError> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;
        Ok(AuthUser {
            address: token_data.claims.sub,
        })
    }

    fn issue_token(&self, address: &str) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.token_ttl_hours);
        let claims = Claims {
            sub: address.to_lowercase(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };
        encode(
            &Header::new(jsonwebtoken::Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(Into::into)
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_challenge_creates_valid_message() {
        let auth = AuthService::new("secret".to_string(), 24);
        let msg = auth
            .generate_challenge("0x0000000000000000000000000000000000000000")
            .unwrap();
        assert!(msg.contains("Sign in to Otter agent"));
        assert!(msg.contains("Nonce:"));
    }

    #[test]
    fn validate_issued_token() {
        let auth = AuthService::new("secret".to_string(), 24);
        let token = auth.issue_token("0xAbC").unwrap();
        let user = auth.validate_token(&token).unwrap();
        assert_eq!(user.address, "0xabc");
    }

    #[test]
    fn verify_signature_accepts_real_eip191_signature() {
        use k256::ecdsa::{RecoveryId, Signature, SigningKey};
        use sha3::{Digest, Keccak256};

        // Well-known Anvil test account #0 private key.
        let private_key =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();
        let signing_key = SigningKey::from_slice(&private_key).unwrap();
        let encoded = signing_key.verifying_key().to_encoded_point(false);
        let digest = Keccak256::digest(&encoded.as_bytes()[1..]);
        let address_bytes: [u8; 20] = digest[12..].try_into().unwrap();
        let address = siwe::eip55(&address_bytes);

        let auth = AuthService::new("secret".to_string(), 24);
        let message = auth.generate_challenge(&address).unwrap();

        // EIP-191 personal_sign over the challenge message.
        let prefixed = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        let prehash = Keccak256::digest(prefixed.as_bytes());
        let (signature, recovery_id): (Signature, RecoveryId) =
            signing_key.sign_prehash_recoverable(&prehash).unwrap();
        let mut signature_bytes = signature.to_bytes().to_vec();
        signature_bytes.push(recovery_id.to_byte() + 27);
        let signature_hex = format!("0x{}", hex::encode(signature_bytes));

        let token = auth.verify_signature(&message, &signature_hex).unwrap();
        let user = auth.validate_token(&token).unwrap();
        assert_eq!(user.address, address.to_lowercase());

        // The challenge is single-use.
        assert!(auth.verify_signature(&message, &signature_hex).is_err());
    }
}
