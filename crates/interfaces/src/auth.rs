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
    #[serde(default)]
    pub role: Role,
}

/// Authorization role for multi-user deployments.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    /// Full control: strategy management, execution toggle, role assignment.
    #[default]
    Owner,
    /// Can create/modify intents and strategies but cannot change roles.
    Admin,
    /// Read-only access to dashboards and status endpoints.
    Viewer,
}

impl Role {
    /// Whether this role is allowed to mutate strategies/intents.
    pub fn can_write(&self) -> bool {
        matches!(self, Role::Owner | Role::Admin)
    }

    pub fn can_manage_roles(&self) -> bool {
        matches!(self, Role::Owner)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Owner => write!(f, "owner"),
            Role::Admin => write!(f, "admin"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Role::Owner),
            "admin" => Ok(Role::Admin),
            "viewer" => Ok(Role::Viewer),
            _ => Err(format!("unknown role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    role: Role,
    /// "access" for API tokens, "refresh" for long-lived refresh tokens.
    kind: String,
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
    /// Address that always receives the `Owner` role.
    owner_address: Option<String>,
    /// Extra role assignments beyond the default Viewer and the configured owner.
    roles: Mutex<HashMap<String, Role>>,
    token_ttl_hours: i64,
    refresh_ttl_days: i64,
}

impl AuthService {
    /// Create a new auth service. If `jwt_secret` is empty, a random one is generated
    /// (fine for dev, NOT for production).
    pub fn new(jwt_secret: String, token_ttl_hours: i64, owner_address: Option<String>) -> Self {
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
            owner_address: owner_address.map(|a| a.to_lowercase()),
            roles: Mutex::new(HashMap::new()),
            token_ttl_hours,
            refresh_ttl_days: 30,
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

    /// Verify a signed SIWE message and return access + refresh tokens on success.
    pub fn verify_signature(
        &self,
        message: &str,
        signature_hex: &str,
    ) -> Result<(String, String), AuthError> {
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
        let role = self.role_of(&address);
        let access = self.issue_access_token(&address, role)?;
        let refresh = self.issue_refresh_token(&address)?;
        Ok((access, refresh))
    }

    /// Resolve the role for an address: the configured owner is always Owner,
    /// otherwise look up an explicit assignment, otherwise default to Viewer.
    pub fn role_of(&self, address: &str) -> Role {
        let normalized = address.to_lowercase();
        if self
            .owner_address
            .as_ref()
            .map(|owner| owner == &normalized)
            .unwrap_or(false)
        {
            return Role::Owner;
        }
        self.roles
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&normalized)
            .copied()
            .unwrap_or(Role::Viewer)
    }

    /// Promote or demote an address. Only the configured owner can assign roles.
    pub fn set_role(&self, caller: &str, address: &str, role: Role) -> Result<(), AuthError> {
        let caller_role = self.role_of(caller);
        if !caller_role.can_manage_roles() {
            return Err(AuthError::VerificationFailed(
                "only the owner can manage roles".to_string(),
            ));
        }
        self.roles
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(address.to_lowercase(), role);
        Ok(())
    }

    /// Validate an access token and return the authenticated user.
    pub fn validate_token(&self, token: &str) -> Result<AuthUser, AuthError> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;
        if token_data.claims.kind != "access" {
            return Err(AuthError::VerificationFailed(
                "token is not an access token".to_string(),
            ));
        }
        Ok(AuthUser {
            address: token_data.claims.sub,
            role: token_data.claims.role,
        })
    }

    /// Exchange a refresh token for a new access token.
    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String, AuthError> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let token_data = decode::<Claims>(
            refresh_token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;
        if token_data.claims.kind != "refresh" {
            return Err(AuthError::VerificationFailed(
                "token is not a refresh token".to_string(),
            ));
        }
        let role = self.role_of(&token_data.claims.sub);
        self.issue_access_token(&token_data.claims.sub, role)
    }

    fn issue_access_token(&self, address: &str, role: Role) -> Result<String, AuthError> {
        self.issue_token(address, role, "access", self.token_ttl_hours)
    }

    fn issue_refresh_token(&self, address: &str) -> Result<String, AuthError> {
        self.issue_token(address, Role::Viewer, "refresh", self.refresh_ttl_days * 24)
    }

    fn issue_token(
        &self,
        address: &str,
        role: Role,
        kind: &str,
        ttl_hours: i64,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(ttl_hours);
        let claims = Claims {
            sub: address.to_lowercase(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            role,
            kind: kind.to_string(),
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
        let auth = AuthService::new("secret".to_string(), 24, None);
        let msg = auth
            .generate_challenge("0x0000000000000000000000000000000000000000")
            .unwrap();
        assert!(msg.contains("Sign in to Otter agent"));
        assert!(msg.contains("Nonce:"));
    }

    #[test]
    fn validate_issued_token() {
        let auth = AuthService::new("secret".to_string(), 24, None);
        let token = auth.issue_access_token("0xAbC", Role::Admin).unwrap();
        let user = auth.validate_token(&token).unwrap();
        assert_eq!(user.address, "0xabc");
        assert_eq!(user.role, Role::Admin);
    }

    #[test]
    fn refresh_token_round_trips() {
        let auth = AuthService::new("secret".to_string(), 24, None);
        let refresh = auth.issue_refresh_token("0xabc").unwrap();
        let access = auth.refresh_access_token(&refresh).unwrap();
        let user = auth.validate_token(&access).unwrap();
        assert_eq!(user.address, "0xabc");
    }

    #[test]
    fn configured_owner_gets_owner_role() {
        let auth = AuthService::new("secret".to_string(), 24, Some("0xOwner".to_string()));
        assert_eq!(auth.role_of("0xowner"), Role::Owner);
        assert_eq!(auth.role_of("0xother"), Role::Viewer);
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

        let auth = AuthService::new("secret".to_string(), 24, None);
        let message = auth.generate_challenge(&address).unwrap();

        // EIP-191 personal_sign over the challenge message.
        let prefixed = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        let prehash = Keccak256::digest(prefixed.as_bytes());
        let (signature, recovery_id): (Signature, RecoveryId) =
            signing_key.sign_prehash_recoverable(&prehash).unwrap();
        let mut signature_bytes = signature.to_bytes().to_vec();
        signature_bytes.push(recovery_id.to_byte() + 27);
        let signature_hex = format!("0x{}", hex::encode(signature_bytes));

        let (access, refresh) = auth.verify_signature(&message, &signature_hex).unwrap();
        let user = auth.validate_token(&access).unwrap();
        assert_eq!(user.address, address.to_lowercase());

        // Refresh token can be exchanged for a new access token.
        let new_access = auth.refresh_access_token(&refresh).unwrap();
        assert!(auth.validate_token(&new_access).is_ok());

        // The challenge is single-use.
        assert!(auth.verify_signature(&message, &signature_hex).is_err());
    }
}
