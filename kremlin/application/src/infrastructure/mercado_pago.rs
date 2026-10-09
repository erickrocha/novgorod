use business::gateway::payment_provider_gateway::{ChargeRequest, PaymentProviderGateway, ProviderCardMetadata, ProviderError, ProviderResult, ProviderStatus};
use business::sea_orm::prelude::async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

pub struct MercadoPago {
    client: reqwest::Client,
    access_token: String,
    pub collector_id: i64,
    /// `MP_API_BASE_URL` overrides the real API, so tests can point it at a provider double.
    base_url: String,
}

#[derive(Deserialize, Default)]
struct CardHolder {
    name: Option<String>,
}

#[derive(Deserialize, Default)]
struct CardResponse {
    last_four_digits: Option<String>,
    expiration_month: Option<i32>,
    expiration_year: Option<i32>,
    cardholder: Option<CardHolder>,
}

#[derive(Deserialize)]
struct PaymentResponse {
    id: serde_json::Value,
    status: String,
    transaction_amount: f64,
    currency_id: String,
    external_reference: Option<String>,
    collector_id: i64,
    payment_type_id: String,
    payment_method_id: Option<String>,
    card: Option<CardResponse>,
}

impl MercadoPago {
    /// Tenant-owned credentials; a missing collector id disables the collector check (0).
    pub fn with_credentials(access_token: String, collector_id: Option<i64>) -> Result<Self, ProviderError> {
        if access_token.trim().is_empty() { return Err(ProviderError("Mercado Pago não configurado".into())); }
        let base_url = std::env::var("MP_API_BASE_URL").ok().filter(|u| !u.is_empty())
            .unwrap_or_else(|| "https://api.mercadopago.com".to_string());
        Ok(Self { client: reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).build()
            .map_err(|e| ProviderError(e.to_string()))?, access_token, collector_id: collector_id.unwrap_or(0), base_url })
    }

    fn normalized_response(response: PaymentResponse) -> Result<ProviderResult, ProviderError> {
        let reference = response.id.as_str().map(str::to_owned).or_else(|| response.id.as_i64().map(|id| id.to_string()))
            .ok_or_else(|| ProviderError("invalid provider reference".into()))?;
        if !response.transaction_amount.is_finite() || response.transaction_amount < 0.0 { return Err(ProviderError("invalid provider amount".into())); }
        let status = match response.status.as_str() {
            "approved" => ProviderStatus::Captured,
            "authorized" => ProviderStatus::Authorized,
            "rejected" | "cancelled" | "refunded" | "charged_back" => ProviderStatus::Failed,
            _ => ProviderStatus::Pending,
        };
        let card = response.card.map(|c| ProviderCardMetadata {
            brand: response.payment_method_id,
            last_four_digits: c.last_four_digits,
            expiration_month: c.expiration_month,
            expiration_year: c.expiration_year,
            cardholder_name: c.cardholder.and_then(|h| h.name),
        });
        Ok(ProviderResult { reference, status, amount_cents: (response.transaction_amount * 100.0).round() as i64,
            currency: response.currency_id, external_reference: response.external_reference.unwrap_or_default(),
            collector_id: response.collector_id, payment_type_id: response.payment_type_id, card })
    }

    fn normalized(&self, response: PaymentResponse) -> Result<ProviderResult, ProviderError> {
        Self::normalized_response(response)
    }

    pub async fn search(&self, external_reference: &str) -> Result<Option<ProviderResult>, ProviderError> {
        let response = self.client.get(format!("{}/v1/payments/search", self.base_url))
            .bearer_auth(&self.access_token).query(&[("external_reference", external_reference)])
            .send().await.map_err(|e| ProviderError(e.to_string()))?;
        if !response.status().is_success() { return Err(ProviderError(format!("provider search returned {}", response.status()))); }
        let body: serde_json::Value = response.json().await.map_err(|e| ProviderError(e.to_string()))?;
        let rows = body.get("results").and_then(|v| v.as_array()).ok_or_else(|| ProviderError("invalid provider search".into()))?;
        if rows.len() > 1 { return Err(ProviderError("ambiguous provider payments".into())); }
        rows.first().map(|v| serde_json::from_value(v.clone()).map_err(|e| ProviderError(e.to_string()))
            .and_then(|r| self.normalized(r))).transpose()
    }
}

#[async_trait]
impl PaymentProviderGateway for MercadoPago {
    async fn charge(&self, request: ChargeRequest) -> Result<ProviderResult, ProviderError> {
        let body = json!({
            "transaction_amount": request.amount_cents as f64 / 100.0,
            "token": request.provider_token,
            "description": format!("Compra Torg {}", request.external_reference),
            "installments": request.installments,
            "payment_method_id": request.payment_method_id,
            "issuer_id": request.issuer_id,
            "external_reference": request.external_reference,
            "payer": {"email": request.payer_email, "identification": {"type": "CPF", "number": request.payer_tax_id}}
        });
        let response = self.client.post(format!("{}/v1/payments", self.base_url))
            .bearer_auth(&self.access_token).header("X-Idempotency-Key", request.idempotency_key)
            .json(&body).send().await.map_err(|e| ProviderError(e.to_string()))?;
        if !response.status().is_success() { return Err(ProviderError(format!("provider charge returned {}", response.status()))); }
        let body = response.json().await.map_err(|e| ProviderError(e.to_string()))?;
        self.normalized(body)
    }

    async fn status(&self, reference: &str) -> Result<ProviderResult, ProviderError> {
        if !reference.bytes().all(|c| c.is_ascii_digit()) { return Err(ProviderError("invalid provider reference".into())); }
        let url = format!("{}/v1/payments/{reference}", self.base_url);
        let response = self.client.get(url).bearer_auth(&self.access_token).send().await.map_err(|e| ProviderError(e.to_string()))?;
        if !response.status().is_success() { return Err(ProviderError(format!("provider status returned {}", response.status()))); }
        let body = response.json().await.map_err(|e| ProviderError(e.to_string()))?;
        self.normalized(body)
    }

    async fn search(&self, external_reference: &str) -> Result<Option<ProviderResult>, ProviderError> {
        self.search(external_reference).await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/infrastructure/mercado_pago_tests.rs"]
mod tests;
