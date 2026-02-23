// OpenClaw Protocol v3 frame types

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Client ID used in the connect handshake.
/// Must be one of the allowed GATEWAY_CLIENT_IDS defined in OpenClaw protocol.
/// "gateway-client" is the generic client ID that doesn't trigger controlUi origin checks.
pub const CLIENT_ID: &str = "gateway-client";
/// Client mode used in the connect handshake
pub const CLIENT_MODE: &str = "ui";
/// Role used in the connect handshake
pub const CLIENT_ROLE: &str = "operator";
/// Scopes requested in the connect handshake
pub const CLIENT_SCOPES: &[&str] = &["operator.admin"];

/// Request frame sent to Gateway
#[derive(Debug, Serialize)]
pub struct RequestFrame {
    #[serde(rename = "type")]
    pub frame_type: String,
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Response frame received from Gateway
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ResponseFrame {
    #[serde(rename = "type")]
    pub frame_type: String,
    pub id: String,
    #[serde(default)]
    pub ok: Option<bool>,
    pub payload: Option<Value>,
    pub error: Option<Value>,
}

/// Event frame received from Gateway (push events forwarded via app.emit)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct EventFrame {
    #[serde(rename = "type")]
    pub frame_type: String,
    pub event: String,
    pub payload: Option<Value>,
}

/// Device authentication info included in the connect frame
pub struct DeviceAuthInfo {
    pub device_id: String,
    /// Raw 32-byte Ed25519 public key, base64url-encoded (no padding)
    pub public_key: String,
    /// Ed25519 signature of the payload string, base64url-encoded (no padding)
    pub signature: String,
    /// Timestamp in milliseconds when the payload was signed
    pub signed_at_ms: i64,
    /// Nonce from the connect.challenge event (optional for local connections)
    pub nonce: Option<String>,
}

/// Build the connect handshake frame for OpenClaw Protocol v3
/// with Ed25519 device authentication.
pub fn build_connect_frame(token: &str, device: Option<DeviceAuthInfo>) -> RequestFrame {
    let connect_id = format!("connect-{}", chrono::Utc::now().timestamp_millis());

    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        "linux"
    };

    let mut params = serde_json::json!({
        "minProtocol": 3,
        "maxProtocol": 3,
        "client": {
            "id": CLIENT_ID,
            "displayName": "ClawX",
            "version": env!("CARGO_PKG_VERSION"),
            "platform": platform,
            "mode": CLIENT_MODE,
        },
        "auth": {
            "token": token,
        },
        "caps": ["tool-events"],
        "role": CLIENT_ROLE,
        "scopes": CLIENT_SCOPES,
    });

    if let Some(d) = device {
        let mut device_obj = serde_json::json!({
            "id": d.device_id,
            "publicKey": d.public_key,
            "signature": d.signature,
            "signedAt": d.signed_at_ms,
        });
        if let Some(nonce) = d.nonce {
            device_obj["nonce"] = serde_json::json!(nonce);
        }
        params["device"] = device_obj;
    }

    RequestFrame {
        frame_type: "req".to_string(),
        id: connect_id,
        method: "connect".to_string(),
        params: Some(params),
    }
}

/// Build an RPC request frame
pub fn build_rpc_frame(method: &str, params: Option<Value>) -> RequestFrame {
    RequestFrame {
        frame_type: "req".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        method: method.to_string(),
        params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_connect_frame_without_device() {
        let frame = build_connect_frame("test-token", None);
        assert_eq!(frame.frame_type, "req");
        assert_eq!(frame.method, "connect");
        assert!(frame.id.starts_with("connect-"));

        let params = frame.params.unwrap();
        assert_eq!(params["auth"]["token"], "test-token");
        assert_eq!(params["minProtocol"], 3);
        assert_eq!(params["client"]["id"], "gateway-client");
        assert!(params.get("device").is_none());
    }

    #[test]
    fn test_build_connect_frame_with_device() {
        let device = DeviceAuthInfo {
            device_id: "abc123".to_string(),
            public_key: "pubkey_b64url".to_string(),
            signature: "sig_b64url".to_string(),
            signed_at_ms: 1234567890,
            nonce: Some("test-nonce".to_string()),
        };
        let frame = build_connect_frame("test-token", Some(device));
        let params = frame.params.unwrap();

        assert_eq!(params["device"]["id"], "abc123");
        assert_eq!(params["device"]["publicKey"], "pubkey_b64url");
        assert_eq!(params["device"]["signature"], "sig_b64url");
        assert_eq!(params["device"]["signedAt"], 1234567890);
        assert_eq!(params["device"]["nonce"], "test-nonce");
    }

    #[test]
    fn test_build_connect_frame_device_no_nonce() {
        let device = DeviceAuthInfo {
            device_id: "abc123".to_string(),
            public_key: "pubkey_b64url".to_string(),
            signature: "sig_b64url".to_string(),
            signed_at_ms: 1234567890,
            nonce: None,
        };
        let frame = build_connect_frame("test-token", Some(device));
        let params = frame.params.unwrap();

        assert_eq!(params["device"]["id"], "abc123");
        assert!(params["device"].get("nonce").is_none());
    }

    #[test]
    fn test_build_rpc_frame() {
        let frame = build_rpc_frame(
            "cron.list",
            Some(serde_json::json!({"includeDisabled": true})),
        );
        assert_eq!(frame.frame_type, "req");
        assert_eq!(frame.method, "cron.list");
        assert!(!frame.id.is_empty());
    }

    #[test]
    fn test_response_frame_deserialize() {
        let json = r#"{"type":"res","id":"abc","ok":true,"payload":{"jobs":[]}}"#;
        let frame: ResponseFrame = serde_json::from_str(json).unwrap();
        assert_eq!(frame.frame_type, "res");
        assert_eq!(frame.id, "abc");
        assert_eq!(frame.ok, Some(true));
    }
}
