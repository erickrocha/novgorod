use business::gateway::payment_provider_gateway::{
    ChargeRequest, PaymentProviderGateway, ProviderCardMetadata, ProviderError, ProviderResult,
    ProviderStatus,
};
use business::sea_orm::prelude::async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

pub struct PagSeguro {
    client: reqwest::Client,
    token: String,
    pub base_url: String,
}

#[derive(Deserialize, Default)]
struct HolderResponse {
    name: Option<String>,
}

#[derive(Deserialize, Default)]
struct CardDetailsResponse {
    brand: Option<String>,
    last_digits: Option<String>,
    exp_month: Option<serde_json::Value>,
    exp_year: Option<serde_json::Value>,
    holder: Option<HolderResponse>,
}

#[derive(Deserialize, Default)]
struct PaymentMethodResponse {
    #[serde(rename = "type")]
    payment_type: Option<String>,
    card: Option<CardDetailsResponse>,
}

#[derive(Deserialize, Default)]
struct AmountResponse {
    value: Option<i64>,
    currency: Option<String>,
}

#[derive(Deserialize)]
struct ChargeResponse {
    id: String,
    reference_id: Option<String>,
    status: String,
    amount: Option<AmountResponse>,
    payment_method: Option<PaymentMethodResponse>,
}

#[derive(Deserialize)]
struct ChargeSearchResponse {
    charges: Option<Vec<ChargeResponse>>,
}

fn parse_int_or_str(v: &serde_json::Value) -> Option<i32> {
    v.as_i64()
        .and_then(|n| i32::try_from(n).ok())
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

impl PagSeguro {
    pub fn with_credentials(
        token: String,
        environment: Option<&str>,
    ) -> Result<Self, ProviderError> {
        if token.is_empty() {
            return Err(ProviderError("Token do PagSeguro inválido".into()));
        }
        let base_url = match environment {
            Some("production") => "https://api.pagseguro.com".to_string(),
            Some("sandbox") => "https://sandbox.api.pagseguro.com".to_string(),
            _ => return Err(ProviderError("Ambiente do PagSeguro inválido".into())),
        };
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| ProviderError(e.to_string()))?;
        Ok(Self {
            client,
            token,
            base_url,
        })
    }

    /// Only provider error codes may be logged; the body can echo card or customer data.
    pub fn error_codes(body: &str) -> String {
        serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| {
                v.get("error_messages").and_then(|m| m.as_array()).map(|rows| {
                    rows.iter()
                        .filter_map(|r| r.get("code").and_then(|c| c.as_str()))
                        .filter(|c| c.len() <= 16 && c.bytes().all(|b| b.is_ascii_alphanumeric()))
                        .collect::<Vec<_>>()
                        .join(",")
                })
            })
            .unwrap_or_default()
    }

    pub fn parse_payment_json(json: serde_json::Value) -> Result<ProviderResult, ProviderError> {
        let response: ChargeResponse =
            serde_json::from_value(json).map_err(|e| ProviderError(e.to_string()))?;
        Self::normalized_response(response)
    }

    fn normalized_response(response: ChargeResponse) -> Result<ProviderResult, ProviderError> {
        let reference = response.id;
        if reference.is_empty() {
            return Err(ProviderError("invalid provider reference".into()));
        }
        let amount_cents = response
            .amount
            .as_ref()
            .and_then(|a| a.value)
            .ok_or_else(|| ProviderError("invalid provider amount".into()))?;
        if amount_cents < 0 {
            return Err(ProviderError("invalid provider amount".into()));
        }
        let currency = response
            .amount
            .as_ref()
            .and_then(|a| a.currency.clone())
            .unwrap_or_else(|| "BRL".into());

        let status = match response.status.to_uppercase().as_str() {
            "PAID" => ProviderStatus::Captured,
            "AUTHORIZED" => ProviderStatus::Authorized,
            "DECLINED" | "CANCELED" => ProviderStatus::Failed,
            _ => ProviderStatus::Pending,
        };

        let card = response.payment_method.as_ref().and_then(|pm| {
            pm.card.as_ref().map(|c| ProviderCardMetadata {
                brand: c.brand.clone(),
                last_four_digits: c.last_digits.clone(),
                expiration_month: c.exp_month.as_ref().and_then(parse_int_or_str),
                expiration_year: c.exp_year.as_ref().and_then(parse_int_or_str),
                cardholder_name: c.holder.as_ref().and_then(|h| h.name.clone()),
            })
        });

        let payment_type_id = response
            .payment_method
            .as_ref()
            .and_then(|pm| pm.payment_type.clone())
            .map(|t| t.to_lowercase())
            .unwrap_or_else(|| "credit_card".into());

        Ok(ProviderResult {
            reference,
            status,
            amount_cents,
            currency,
            external_reference: response.reference_id.unwrap_or_default(),
            collector_id: 0,
            payment_type_id,
            card,
        })
    }
}

#[async_trait]
impl PaymentProviderGateway for PagSeguro {
    async fn charge(&self, request: ChargeRequest) -> Result<ProviderResult, ProviderError> {
        let card_payload = if request.provider_token.starts_with('{') {
            serde_json::from_str::<serde_json::Value>(&request.provider_token)
                .unwrap_or_else(|_| json!({ "encrypted": request.provider_token }))
        } else if request.provider_token.len() > 64 {
            json!({ "encrypted": request.provider_token })
        } else {
            json!({ "token": request.provider_token })
        };

        let body = json!({
            "reference_id": request.external_reference,
            "description": format!("Compra Torg {}", request.external_reference),
            "amount": {
                "value": request.amount_cents,
                "currency": request.currency
            },
            "payment_method": {
                "type": "CREDIT_CARD",
                "installments": request.installments,
                "capture": true,
                "card": card_payload
            },
            "customer": {
                "email": request.payer_email,
                "tax_id": request.payer_tax_id
            }
        });

        let url = format!("{}/charges", self.base_url);
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .header("x-idempotency-key", request.idempotency_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError(format!(
                "pagseguro charge returned {status} codes [{}]",
                Self::error_codes(&error_text)
            )));
        }

        let body = response
            .json()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;
        Self::parse_payment_json(body)
    }

    async fn status(&self, reference: &str) -> Result<ProviderResult, ProviderError> {
        let url = format!("{}/charges/{reference}", self.base_url);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError(format!(
                "pagseguro status returned {}",
                response.status()
            )));
        }

        let body = response
            .json()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;
        Self::parse_payment_json(body)
    }

    async fn cancel(&self, reference: &str) -> Result<ProviderResult, ProviderError> {
        // Cancelling a paid charge is a refund, which is out of scope: refuse it.
        let current = self.status(reference).await?;
        if current.status == ProviderStatus::Captured {
            return Err(ProviderError("payment already captured; cancel refused".into()));
        }
        let url = format!("{}/charges/{reference}/cancel", self.base_url);
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&json!({"amount": {"value": current.amount_cents}}))
            .send()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(ProviderError(format!(
                "pagseguro cancel returned {}",
                response.status()
            )));
        }
        let body = response
            .json()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;
        Self::parse_payment_json(body)
    }

    async fn search(
        &self,
        external_reference: &str,
    ) -> Result<Option<ProviderResult>, ProviderError> {
        let url = format!("{}/charges", self.base_url);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .query(&[("reference_id", external_reference)])
            .send()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError(format!(
                "pagseguro search returned {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError(e.to_string()))?;

        if let Ok(search) = serde_json::from_value::<ChargeSearchResponse>(body.clone())
            && let Some(charges) = search.charges
                && let Some(first) = charges.into_iter().next() {
                    return Self::normalized_response(first).map(Some);
                }

        if let Some(arr) = body.as_array()
            && let Some(first) = arr.first() {
                return Self::parse_payment_json(first.clone()).map(Some);
            }

        Ok(None)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/infrastructure/pagseguro_tests.rs"]
mod tests;
