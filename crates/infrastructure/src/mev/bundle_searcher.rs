//! Real bundle-based MEV searcher (V2) using Flashbots-compatible relays.
//!
//! This module signs and submits bundles to private block builders. It is
//! intentionally decoupled from the V1 simulated-capture accounting: the latter
//! records deterministic rebates, while this module handles on-chain bundle
//! submission. A future version can reconcile the two by parsing the winning
//! bundle's coinbase transfer or post-block balance change.

use alloy::primitives::{Address, B256, keccak256};
use async_trait::async_trait;
use domain::ports::searcher_port::{Bundle, BundleSearcherPort, SearcherError};
use k256::ecdsa::{SigningKey, VerifyingKey};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

/// Flashbots-compatible relay client.
pub struct FlashbotsBundleSearcher {
    client: Client,
    relay_url: Url,
    signing_key: SigningKey,
    searcher_address: Address,
}

impl FlashbotsBundleSearcher {
    /// Create a new searcher from a private-key hex string and a relay URL.
    pub fn new(relay_url: &str, private_key_hex: &str) -> Result<Self, SearcherError> {
        let url = Url::parse(relay_url)
            .map_err(|e| SearcherError::InvalidInput(format!("invalid relay url: {}", e)))?;

        let cleaned = private_key_hex
            .trim()
            .strip_prefix("0x")
            .unwrap_or(private_key_hex);
        let mut key_bytes = [0u8; 32];
        let decoded = hex::decode(cleaned)
            .map_err(|e| SearcherError::InvalidInput(format!("invalid private key hex: {}", e)))?;
        if decoded.len() != 32 {
            return Err(SearcherError::InvalidInput(format!(
                "private key must be 32 bytes, got {}",
                decoded.len()
            )));
        }
        key_bytes.copy_from_slice(&decoded);

        let signing_key = SigningKey::from_slice(&key_bytes)
            .map_err(|e| SearcherError::InvalidInput(format!("invalid private key: {}", e)))?;
        let verifying_key = VerifyingKey::from(&signing_key);
        let searcher_address = address_from_verifying_key(&verifying_key);

        Ok(Self {
            client: Client::new(),
            relay_url: url,
            signing_key,
            searcher_address,
        })
    }
}

#[async_trait]
impl BundleSearcherPort for FlashbotsBundleSearcher {
    async fn submit_bundle(&self, bundle: Bundle) -> Result<String, SearcherError> {
        if bundle.txs.is_empty() {
            return Err(SearcherError::InvalidInput(
                "bundle must contain at least one transaction".to_string(),
            ));
        }

        let txs: Vec<String> = bundle
            .txs
            .iter()
            .map(|tx| format!("0x{}", hex::encode(tx)))
            .collect();

        let params = SendBundleParams {
            txs,
            block_number: bundle.block_number.map(|n| format!("0x{:x}", n)),
            min_timestamp: bundle.min_timestamp,
            max_timestamp: bundle.max_timestamp,
        };

        let body = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "eth_sendBundle".to_string(),
            params: vec![params],
        };

        let body_json = serde_json::to_vec(&body)
            .map_err(|e| SearcherError::SubmissionFailed(format!("serialize: {}", e)))?;
        let body_hash = keccak256(&body_json);
        let signature = sign_relay_message(&self.signing_key, &body_hash)?;
        let auth_header = format!("{}:{}", self.searcher_address, signature);

        let response = self
            .client
            .post(self.relay_url.clone())
            .header("Content-Type", "application/json")
            .header("X-Flashbots-Signature", auth_header)
            .body(body_json)
            .send()
            .await
            .map_err(|e| SearcherError::SubmissionFailed(format!("relay request: {}", e)))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| SearcherError::SubmissionFailed(format!("read body: {}", e)))?;

        if !status.is_success() {
            return Err(SearcherError::SubmissionFailed(format!(
                "relay returned {}: {}",
                status, text
            )));
        }

        let parsed: JsonRpcResponse<SendBundleResult> = serde_json::from_str(&text)
            .map_err(|e| SearcherError::SubmissionFailed(format!("decode: {} ({})", e, text)))?;

        parsed
            .result
            .map(|r| r.bundle_hash)
            .ok_or_else(|| SearcherError::SubmissionFailed(format!("no bundle hash: {}", text)))
    }
}

fn address_from_verifying_key(key: &VerifyingKey) -> Address {
    let encoded = key.to_encoded_point(false);
    let bytes = encoded.as_bytes();
    // Skip the 0x04 uncompressed prefix.
    if bytes.len() != 65 {
        panic!("unexpected verifying key length");
    }
    let mut digest = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(&bytes[1..]);
    hasher.finalize(&mut digest);
    Address::from_slice(&digest[12..])
}

fn sign_relay_message(signing_key: &SigningKey, hash: &B256) -> Result<String, SearcherError> {
    let (sig, recid) = signing_key
        .sign_prehash_recoverable(hash.as_ref())
        .map_err(|e| SearcherError::SubmissionFailed(format!("sign: {}", e)))?;
    let mut out = [0u8; 65];
    out[..64].copy_from_slice(&sig.to_bytes());
    out[64] = recid.to_byte();
    Ok(format!("0x{}", hex::encode(out)))
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T> {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SendBundleParams {
    txs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_timestamp: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendBundleResult {
    bundle_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PK: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    fn searcher_derives_expected_address() {
        let searcher =
            FlashbotsBundleSearcher::new("https://relay.flashbots.net", TEST_PK).unwrap();
        assert_eq!(
            searcher.searcher_address.to_string(),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
    }

    #[tokio::test]
    async fn submit_empty_bundle_fails() {
        let searcher =
            FlashbotsBundleSearcher::new("https://relay.flashbots.net", TEST_PK).unwrap();
        let err = searcher
            .submit_bundle(Bundle {
                txs: vec![],
                block_number: None,
                min_timestamp: None,
                max_timestamp: None,
            })
            .await
            .unwrap_err();
        assert!(err.to_string().contains("at least one transaction"));
    }
}
