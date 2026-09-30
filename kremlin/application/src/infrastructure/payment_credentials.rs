//! Deployment-managed AES-256-GCM key ring for payment credentials.
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, AeadCore, OsRng, Payload},
};
use business::domain::marketplace::PurchaseError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MercadoPagoCredentials {
    #[schema(write_only)]
    pub access_token: String,
    pub public_key: Option<String>,
    pub collector_id: Option<i64>,
    pub webhook_secret: Option<String>,
}

impl std::fmt::Debug for MercadoPagoCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MercadoPagoCredentials([REDACTED])")
    }
}

impl MercadoPagoCredentials {
    pub fn validate(&self) -> Result<(), PurchaseError> {
        if self.access_token.trim().is_empty() || self.access_token.len() > 512 {
            return Err(PurchaseError::Validation("Mercado Pago access token is required"));
        }
        Ok(())
    }
}

#[derive(Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PagSeguroCredentials {
    #[schema(write_only)]
    pub token: String,
    pub public_key: Option<String>,
    pub environment: Option<String>,
}

impl std::fmt::Debug for PagSeguroCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PagSeguroCredentials([REDACTED])")
    }
}

impl PagSeguroCredentials {
    pub fn validate(&self) -> Result<(), PurchaseError> {
        if self.token.trim().is_empty() || self.token.len() > 512 {
            return Err(PurchaseError::Validation("PagSeguro token is required"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum TenantCredentialsPayload {
    MercadoPago(MercadoPagoCredentials),
    PagSeguro(PagSeguroCredentials),
}

#[derive(Default)]
pub struct PaymentKeyRing {
    active: String,
    keys: BTreeMap<String, [u8; 32]>,
}

impl PaymentKeyRing {
    pub fn from_env() -> Result<Self, &'static str> {
        let raw = match std::env::var("PAYMENT_ENCRYPTION_KEYS")
            .or_else(|_| std::env::var("SHIPPING_ENCRYPTION_KEYS"))
        {
            Ok(v) => v,
            Err(std::env::VarError::NotPresent) => return Ok(Self::default()),
            Err(_) => return Err("invalid payment encryption configuration"),
        };
        let values: BTreeMap<String, String> =
            serde_json::from_str(&raw).map_err(|_| "invalid payment encryption keys")?;
        let active = std::env::var("PAYMENT_ACTIVE_KEY_VERSION")
            .or_else(|_| std::env::var("SHIPPING_ACTIVE_KEY_VERSION"))
            .map_err(|_| "payment active key version required")?;
        let mut keys = BTreeMap::new();
        for (version, value) in values {
            if version.is_empty() || version.len() > 100 || value.len() != 64 {
                return Err("invalid payment encryption key");
            }
            let mut key = [0_u8; 32];
            for (i, pair) in value.as_bytes().chunks_exact(2).enumerate() {
                let hex =
                    std::str::from_utf8(pair).map_err(|_| "invalid payment encryption key")?;
                key[i] =
                    u8::from_str_radix(hex, 16).map_err(|_| "invalid payment encryption key")?;
            }
            keys.insert(version, key);
        }
        if !keys.contains_key(&active) {
            return Err("active payment key missing");
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
        format!("payment:{tenant_id}:{version}")
    }

    pub fn encrypt_payload(
        &self,
        tenant_id: i64,
        payload: &TenantCredentialsPayload,
    ) -> Result<(String, Vec<u8>), PurchaseError> {
        let cipher = self.cipher(&self.active)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let plaintext =
            serde_json::to_vec(payload).map_err(|_| PurchaseError::ShippingUnavailable)?;
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

    pub fn decrypt_payload(
        &self,
        tenant_id: i64,
        version: &str,
        bytes: &[u8],
    ) -> Result<TenantCredentialsPayload, PurchaseError> {
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
#[path = "../../tests/unit/infrastructure/payment_credentials_test.rs"]
mod tests;
