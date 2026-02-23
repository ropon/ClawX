// Device Identity — Ed25519 keypair management for Gateway device authentication
//
// Each ClawX installation generates a unique Ed25519 keypair stored in
// clawx-device-identity.json. The public key fingerprint (SHA-256 hex) serves
// as the stable device ID. The Gateway auto-pairs local devices silently.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const IDENTITY_FILE: &str = "clawx-device-identity.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub version: u32,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    /// Raw 32-byte Ed25519 public key, base64url-encoded (no padding)
    #[serde(rename = "publicKey")]
    pub public_key: String,
    /// Raw 32-byte Ed25519 private key, base64url-encoded (no padding)
    #[serde(rename = "privateKey")]
    pub private_key: String,
    #[serde(rename = "createdAtMs")]
    pub created_at_ms: i64,
}

impl DeviceIdentity {
    /// Generate a new device identity with a fresh Ed25519 keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let public_key_bytes = signing_key.verifying_key().to_bytes();
        let private_key_bytes = signing_key.to_bytes();

        let device_id = fingerprint_public_key(&public_key_bytes);
        let public_key = URL_SAFE_NO_PAD.encode(public_key_bytes);
        let private_key = URL_SAFE_NO_PAD.encode(private_key_bytes);

        Self {
            version: 1,
            device_id,
            public_key,
            private_key,
            created_at_ms: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Load existing identity from disk, or generate and persist a new one
    pub fn load_or_create(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join(IDENTITY_FILE);

        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read device identity: {}", e))?;
            let identity: Self = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse device identity: {}", e))?;

            // Validate the identity is usable (can reconstruct the signing key)
            if identity.signing_key().is_ok() {
                return Ok(identity);
            }
            eprintln!("[ClawX] Device identity corrupted, regenerating");
        }

        let identity = Self::generate();
        identity.save(data_dir)?;
        Ok(identity)
    }

    /// Persist identity to disk
    fn save(&self, data_dir: &Path) -> Result<(), String> {
        let path = data_dir.join(IDENTITY_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create identity dir: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize identity: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write identity: {}", e))?;

        // Set file permissions to owner-only (0600) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    /// Reconstruct the Ed25519 signing key from stored private key bytes
    fn signing_key(&self) -> Result<SigningKey, String> {
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.private_key)
            .map_err(|e| format!("Invalid private key encoding: {}", e))?;
        let key_bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "Private key must be 32 bytes".to_string())?;
        Ok(SigningKey::from_bytes(&key_bytes))
    }

    /// Sign the device auth payload and return base64url-encoded signature
    pub fn sign_payload(&self, payload: &str) -> Result<String, String> {
        let signing_key = self.signing_key()?;
        let signature = signing_key.sign(payload.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(signature.to_bytes()))
    }
}

/// Derive device ID from raw 32-byte Ed25519 public key: SHA-256 hex
fn fingerprint_public_key(raw_bytes: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_bytes);
    let hash = hasher.finalize();
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Build the pipe-delimited payload string for device auth signing.
///
/// v1: `v1|{deviceId}|{clientId}|{clientMode}|{role}|{scopes}|{signedAtMs}|{token}`
/// v2: `v2|{deviceId}|{clientId}|{clientMode}|{role}|{scopes}|{signedAtMs}|{token}|{nonce}`
pub fn build_device_auth_payload(
    device_id: &str,
    client_id: &str,
    client_mode: &str,
    role: &str,
    scopes: &[&str],
    signed_at_ms: i64,
    token: &str,
    nonce: Option<&str>,
) -> String {
    let version = if nonce.is_some() { "v2" } else { "v1" };
    let scopes_str = scopes.join(",");

    let mut parts = vec![
        version.to_string(),
        device_id.to_string(),
        client_id.to_string(),
        client_mode.to_string(),
        role.to_string(),
        scopes_str,
        signed_at_ms.to_string(),
        token.to_string(),
    ];

    if let Some(n) = nonce {
        parts.push(n.to_string());
    }

    parts.join("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let identity = DeviceIdentity::generate();
        assert_eq!(identity.version, 1);
        assert_eq!(identity.device_id.len(), 64); // SHA-256 hex = 64 chars
        assert!(!identity.public_key.is_empty());
        assert!(!identity.private_key.is_empty());
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let identity = DeviceIdentity::generate();
        let payload =
            "v2|test-device|test-client|ui|operator|operator.admin|1234567890|token|nonce";
        let signature = identity.sign_payload(payload).unwrap();
        assert!(!signature.is_empty());

        // Verify: decode public key and signature, use ed25519_dalek to verify
        let pub_bytes = URL_SAFE_NO_PAD.decode(&identity.public_key).unwrap();
        let pub_key_bytes: [u8; 32] = pub_bytes.try_into().unwrap();
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pub_key_bytes).unwrap();

        let sig_bytes = URL_SAFE_NO_PAD.decode(&signature).unwrap();
        let sig_array: [u8; 64] = sig_bytes.try_into().unwrap();
        let sig = ed25519_dalek::Signature::from_bytes(&sig_array);

        use ed25519_dalek::Verifier;
        assert!(verifying_key.verify(payload.as_bytes(), &sig).is_ok());
    }

    #[test]
    fn test_device_id_is_sha256_of_public_key() {
        let identity = DeviceIdentity::generate();
        let pub_bytes = URL_SAFE_NO_PAD.decode(&identity.public_key).unwrap();
        let pub_key_bytes: [u8; 32] = pub_bytes.try_into().unwrap();
        let expected_id = fingerprint_public_key(&pub_key_bytes);
        assert_eq!(identity.device_id, expected_id);
    }

    #[test]
    fn test_build_payload_v1() {
        let payload = build_device_auth_payload(
            "abc123",
            "client-id",
            "ui",
            "operator",
            &["operator.admin"],
            999,
            "token",
            None,
        );
        assert_eq!(
            payload,
            "v1|abc123|client-id|ui|operator|operator.admin|999|token"
        );
    }

    #[test]
    fn test_build_payload_v2() {
        let payload = build_device_auth_payload(
            "abc123",
            "client-id",
            "ui",
            "operator",
            &["operator.admin", "operator.write"],
            999,
            "token",
            Some("nonce-uuid"),
        );
        assert_eq!(
            payload,
            "v2|abc123|client-id|ui|operator|operator.admin,operator.write|999|token|nonce-uuid"
        );
    }

    #[test]
    fn test_load_or_create_new() {
        let dir = tempfile::tempdir().unwrap();
        let identity = DeviceIdentity::load_or_create(dir.path()).unwrap();
        assert_eq!(identity.version, 1);
        assert_eq!(identity.device_id.len(), 64);

        // Should persist to disk
        let path = dir.path().join(IDENTITY_FILE);
        assert!(path.exists());

        // Loading again should return the same identity
        let identity2 = DeviceIdentity::load_or_create(dir.path()).unwrap();
        assert_eq!(identity.device_id, identity2.device_id);
    }
}
