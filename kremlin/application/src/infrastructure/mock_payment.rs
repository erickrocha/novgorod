use business::gateway::payment_provider_gateway::{
    ChargeRequest, PaymentProviderGateway, ProviderCardMetadata, ProviderError, ProviderResult,
    ProviderStatus,
};
use business::sea_orm::prelude::async_trait::async_trait;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MockProviderKind {
    MercadoPago,
    PagSeguro,
}

impl MockProviderKind {
    pub fn parse(value: &str) -> Result<Self, ProviderError> {
        match value {
            "mercado_pago" => Ok(Self::MercadoPago),
            "pagseguro" => Ok(Self::PagSeguro),
            _ => Err(ProviderError(
                "PAYMENT_MOCK_PROVIDER must be mercado_pago or pagseguro".into(),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MercadoPago => "mercado_pago",
            Self::PagSeguro => "pagseguro",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MockOutcome {
    Captured,
    Authorized,
    Pending,
    Failed,
    ProviderError,
}

impl MockOutcome {
    fn parse(value: &str) -> Result<Self, ProviderError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "captured" | "approved" | "paid" | "apro" => Ok(Self::Captured),
            "authorized" | "authorised" | "cont" => Ok(Self::Authorized),
            "pending" | "waiting" | "in_analysis" | "in_process" | "othe" => {
                Ok(Self::Pending)
            }
            "failed" | "declined" | "rejected" | "canceled" | "cancelled" | "fund" | "call" => {
                Ok(Self::Failed)
            }
            "error" | "provider_error" | "network_error" => Ok(Self::ProviderError),
            _ => Err(ProviderError(
                "mock token must select captured, authorized, pending, failed, or error".into(),
            )),
        }
    }

    fn result_status(self) -> Result<ProviderStatus, ProviderError> {
        match self {
            Self::Captured => Ok(ProviderStatus::Captured),
            Self::Authorized => Ok(ProviderStatus::Authorized),
            Self::Pending => Ok(ProviderStatus::Pending),
            Self::Failed => Ok(ProviderStatus::Failed),
            Self::ProviderError => Err(ProviderError("mock provider error requested".into())),
        }
    }

    fn reference_status(self) -> &'static str {
        match self {
            Self::Captured => "captured",
            Self::Authorized => "authorized",
            Self::Pending => "pending",
            Self::Failed => "failed",
            Self::ProviderError => "error",
        }
    }
}

pub struct MockPaymentProvider {
    kind: MockProviderKind,
}

impl MockPaymentProvider {
    pub fn configured() -> Result<Self, ProviderError> {
        let provider = std::env::var("PAYMENT_MOCK_PROVIDER")
            .map_err(|_| ProviderError("PAYMENT_MOCK_PROVIDER is required in mock mode".into()))?;
        Ok(Self {
            kind: MockProviderKind::parse(&provider)?,
        })
    }

    pub fn new(kind: MockProviderKind) -> Self {
        Self { kind }
    }

    pub fn name(&self) -> &'static str {
        self.kind.as_str()
    }

    fn outcome_from_payload(payload: &str) -> Result<MockOutcome, ProviderError> {
        if let Some(outcome) = payload.strip_prefix("mock:") {
            return MockOutcome::parse(outcome);
        }

        let value: Value = serde_json::from_str(payload)
            .map_err(|_| ProviderError("mock token must be prefixed with mock:".into()))?;
        let outcome = value
            .get("mockOutcome")
            .or_else(|| value.get("mockStatus"))
            .and_then(Value::as_str)
            .ok_or_else(|| ProviderError("mock payload needs mockOutcome".into()))?;
        MockOutcome::parse(outcome)
    }

    fn result(
        &self,
        outcome: MockOutcome,
        amount_cents: i64,
        currency: String,
        external_reference: String,
        payment_method_id: String,
    ) -> Result<ProviderResult, ProviderError> {
        let status = outcome.result_status()?;
        let reference = format!(
            "mock:{}:{}:{}:{}:{}",
            self.name(),
            outcome.reference_status(),
            amount_cents,
            currency,
            external_reference
        );
        Ok(ProviderResult {
            reference,
            status,
            amount_cents,
            currency,
            external_reference,
            collector_id: 0,
            payment_type_id: "credit_card".into(),
            card: Some(ProviderCardMetadata {
                brand: Some(payment_method_id),
                last_four_digits: Some("4242".into()),
                expiration_month: Some(12),
                expiration_year: Some(2030),
                cardholder_name: Some("Mock Test Card".into()),
            }),
        })
    }

    fn result_from_reference(&self, reference: &str) -> Result<ProviderResult, ProviderError> {
        let parts: Vec<_> = reference.splitn(6, ':').collect();
        if parts.len() != 6 || parts[0] != "mock" || parts[1] != self.name() {
            return Err(ProviderError("invalid mock payment reference".into()));
        }
        let outcome = MockOutcome::parse(parts[2])?;
        let amount_cents = parts[3]
            .parse::<i64>()
            .map_err(|_| ProviderError("invalid mock payment amount".into()))?;
        if amount_cents < 0 {
            return Err(ProviderError("invalid mock payment amount".into()));
        }
        self.result(
            outcome,
            amount_cents,
            parts[4].to_string(),
            parts[5].to_string(),
            "mock".into(),
        )
    }
}

#[async_trait]
impl PaymentProviderGateway for MockPaymentProvider {
    async fn charge(&self, request: ChargeRequest) -> Result<ProviderResult, ProviderError> {
        let outcome = Self::outcome_from_payload(&request.provider_token)?;
        self.result(
            outcome,
            request.amount_cents,
            request.currency,
            request.external_reference,
            request.payment_method_id,
        )
    }

    async fn status(&self, reference: &str) -> Result<ProviderResult, ProviderError> {
        self.result_from_reference(reference)
    }

    async fn search(
        &self,
        _external_reference: &str,
    ) -> Result<Option<ProviderResult>, ProviderError> {
        Ok(None)
    }

    async fn cancel(&self, reference: &str) -> Result<ProviderResult, ProviderError> {
        let current = self.result_from_reference(reference)?;
        if current.status == ProviderStatus::Captured {
            return Err(ProviderError("payment already captured; cancel refused".into()));
        }
        Ok(ProviderResult {
            status: ProviderStatus::Failed,
            ..current
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/infrastructure/mock_payment_provider_test.rs"]
mod tests;