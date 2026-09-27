//! Deployment-managed AES-256-GCM key ring. Never log keys or plaintext credentials.
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, AeadCore, OsRng, Payload},
};
use business::domain::marketplace::PurchaseError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorreiosCredentials {
    #[schema(write_only)]
    pub username: String,
    #[schema(write_only)]
    pub api_access_code: String,
    #[schema(write_only)]
    pub posting_card: String,
    #[schema(write_only)]
    pub contract: String,
    #[schema(write_only)]
    pub regional_identifier: String,
}
impl std::fmt::Debug for CorreiosCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CorreiosCredentials([REDACTED])")
    }
}
impl CorreiosCredentials {
    pub fn validate(&self) -> Result<(), PurchaseError> {
        if [
            &self.username,
            &self.api_access_code,
            &self.posting_card,
            &self.contract,
            &self.regional_identifier,
        ]
        .iter()
        .any(|v| v.trim().is_empty() || v.len() > 256 || v.chars().any(char::is_control))
        {
            return Err(PurchaseError::Validation(
                "complete Correios credentials required",
            ));
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct ShippingKeyRing {
    active: String,
    keys: BTreeMap<String, [u8; 32]>,
}
impl ShippingKeyRing {
    pub fn from_env() -> Result<Self, &'static str> {
        let raw = match std::env::var("SHIPPING_ENCRYPTION_KEYS") {
            Ok(v) => v,
            Err(std::env::VarError::NotPresent) => return Ok(Self::default()),
            Err(_) => return Err("invalid shipping encryption configuration"),
        };
        let values: BTreeMap<String, String> =
            serde_json::from_str(&raw).map_err(|_| "invalid shipping encryption keys")?;
        let active = std::env::var("SHIPPING_ACTIVE_KEY_VERSION")
            .map_err(|_| "shipping active key version required")?;
        let mut keys = BTreeMap::new();
        for (version, value) in values {
            if version.is_empty() || version.len() > 100 || value.len() != 64 {
                return Err("invalid shipping encryption key");
            }
            let mut key = [0_u8; 32];
            for (i, pair) in value.as_bytes().chunks_exact(2).enumerate() {
                let hex =
                    std::str::from_utf8(pair).map_err(|_| "invalid shipping encryption key")?;
                key[i] =
                    u8::from_str_radix(hex, 16).map_err(|_| "invalid shipping encryption key")?;
            }
            keys.insert(version, key);
        }
        if !keys.contains_key(&active) {
            return Err("active shipping key missing");
        }
        Ok(Self { active, keys })
    }
    fn cipher(&self, version: &str) -> Result<Aes256Gcm, PurchaseError> {
        let key = self
            .keys
            .get(version)
            .ok_or(PurchaseError::ShippingUnavailable)?;
        Aes256Gcm::new_from_slice(key).map_err(|_| PurchaseError::ShippingUnavailable)
    }
    fn aad(tenant_id: i64, version: &str) -> String {
        format!("shipping:correios:{tenant_id}:{version}")
    }
    pub fn encrypt(
        &self,
        tenant_id: i64,
        credentials: &CorreiosCredentials,
    ) -> Result<(String, Vec<u8>), PurchaseError> {
        let cipher = self.cipher(&self.active)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let plaintext =
            serde_json::to_vec(credentials).map_err(|_| PurchaseError::ShippingUnavailable)?;
        let aad = Self::aad(tenant_id, &self.active);
        let encrypted = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: &plaintext,
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| PurchaseError::ShippingUnavailable)?;
        let mut bytes = nonce.to_vec();
        bytes.extend(encrypted);
        Ok((self.active.clone(), bytes))
    }
    pub fn decrypt(
        &self,
        tenant_id: i64,
        version: &str,
        bytes: &[u8],
    ) -> Result<CorreiosCredentials, PurchaseError> {
        if bytes.len() < 28 {
            return Err(PurchaseError::ShippingUnavailable);
        }
        let cipher = self.cipher(version)?;
        let aad = Self::aad(tenant_id, version);
        let nonce: [u8; 12] = bytes[..12]
            .try_into()
            .map_err(|_| PurchaseError::ShippingUnavailable)?;
        let plaintext = cipher
            .decrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: &bytes[12..],
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| PurchaseError::ShippingUnavailable)?;
        serde_json::from_slice(&plaintext).map_err(|_| PurchaseError::ShippingUnavailable)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encryption_is_random_authenticated_and_tenant_bound() {
        let ring = ShippingKeyRing {
            active: "v1".into(),
            keys: BTreeMap::from([("v1".into(), [42; 32])]),
        };
        let credentials = CorreiosCredentials {
            username: "secret-user".into(),
            api_access_code: "secret-code".into(),
            ..Default::default()
        };
        let (version, mut bytes) = ring.encrypt(7, &credentials).unwrap();
        assert_ne!(bytes, ring.encrypt(7, &credentials).unwrap().1);
        assert_eq!(
            ring.decrypt(7, &version, &bytes).unwrap().username,
            "secret-user"
        );
        assert!(ring.decrypt(8, &version, &bytes).is_err());
        assert!(ring.decrypt(7, "v2", &bytes).is_err());
        bytes[15] ^= 1;
        assert!(ring.decrypt(7, &version, &bytes).is_err());
        assert!(!format!("{credentials:?}").contains("secret"));
    }
}
