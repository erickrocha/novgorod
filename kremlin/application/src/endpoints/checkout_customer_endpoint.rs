use crate::AppState;
use crate::commons::{exception_response::{ExceptionResponse, HttpResponse}, i18n::{ErrorKey, Locale}};
use axum::{Json, extract::{Extension, State}};
use business::{domain::{enums::Role, user::User}, sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set}};
use entity::{customer_address_entity, customer_entity};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutAddress {
    id: i64,
    label: Option<String>,
    recipient: String,
    address_line1: Option<String>,
    address_line2: Option<String>,
    locality: Option<String>,
    administrative_area: Option<String>,
    postal_code: Option<String>,
    country_code: Option<String>,
    is_default: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutCustomer {
    id: i64,
    name: String,
    cpf: Option<String>,
    email: String,
    phone: Option<String>,
    addresses: Vec<CheckoutAddress>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaxIdInput { cpf: String }

fn valid_cpf(cpf: &str) -> bool {
    let digits: Vec<u32> = cpf.chars().filter_map(|c| c.to_digit(10)).collect();
    if cpf.chars().any(|c| !c.is_ascii_digit()) || digits.len() != 11 || digits.iter().all(|d| *d == digits[0]) { return false; }
    for index in [9, 10] {
        let sum: u32 = digits[..index].iter().enumerate().map(|(i, d)| d * (index as u32 + 1 - i as u32)).sum();
        let check = (sum * 10) % 11 % 10;
        if check != digits[index] { return false; }
    }
    true
}

async fn owned(state: &AppState, user: &User, locale: Locale) -> Result<customer_entity::Model, ExceptionResponse> {
    if user.role != Role::Customer { return Err(ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden)); }
    let id = user.id.ok_or(ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden))?;
    customer_entity::Entity::find().filter(customer_entity::Column::UserId.eq(id))
        .filter(customer_entity::Column::Active.eq(true)).one(state.conn.as_ref()).await
        .map_err(|_| ExceptionResponse::InternalServerError(locale, ErrorKey::PurchaseUnavailable))?
        .ok_or(ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden))
}

pub async fn me(State(state): State<AppState>, Extension(user): Extension<User>, Extension(locale): Extension<Locale>) -> HttpResponse<Json<CheckoutCustomer>> {
    let customer = owned(&state, &user, locale).await?;
    let addresses = customer_address_entity::Entity::find()
        .filter(customer_address_entity::Column::CustomerId.eq(customer.id))
        .order_by_desc(customer_address_entity::Column::IsDefault)
        .order_by_asc(customer_address_entity::Column::Id)
        .all(state.conn.as_ref()).await
        .map_err(|_| ExceptionResponse::InternalServerError(locale, ErrorKey::PurchaseUnavailable))?
        .into_iter().map(|a| CheckoutAddress { id: a.id, label: a.label, recipient: a.recipient,
            address_line1: a.address_line1, address_line2: a.address_line2, locality: a.locality,
            administrative_area: a.administrative_area, postal_code: a.postal_code,
            country_code: a.country_code, is_default: a.is_default }).collect();
    Ok(Json(CheckoutCustomer { id: customer.id, name: customer.name, cpf: customer.cpf,
        email: customer.email, phone: customer.phone, addresses }))
}

pub async fn complete_tax_id(State(state): State<AppState>, Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>, Json(input): Json<TaxIdInput>) -> HttpResponse<Json<CheckoutCustomer>> {
    let customer = owned(&state, &user, locale).await?;
    if customer.cpf.as_deref().is_some_and(|value| !value.trim().is_empty()) {
        return Err(ExceptionResponse::Conflict(locale, ErrorKey::PurchaseConflict));
    }
    if !valid_cpf(&input.cpf) { return Err(ExceptionResponse::BadRequest(locale, ErrorKey::PurchaseInvalid)); }
    let duplicate = customer_entity::Entity::find().filter(customer_entity::Column::Cpf.eq(&input.cpf))
        .one(state.conn.as_ref()).await.map_err(|_| ExceptionResponse::InternalServerError(locale, ErrorKey::PurchaseUnavailable))?;
    if duplicate.is_some() { return Err(ExceptionResponse::Conflict(locale, ErrorKey::PurchaseConflict)); }
    let mut update: customer_entity::ActiveModel = customer.into();
    update.cpf = Set(Some(input.cpf));
    update.update(state.conn.as_ref()).await.map_err(|_| ExceptionResponse::Conflict(locale, ErrorKey::PurchaseConflict))?;
    me(State(state), Extension(user), Extension(locale)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use business::domain::shipping::ShippingOption;
    use business::gateway::shipping_provider_gateway::{
        ShippingProviderError, ShippingProviderGateway, ShippingRequest,
    };
    use business::domain::enums::Role;
    use business::sea_orm::{DatabaseBackend, DbConn, MockDatabase};
    use chrono::Utc;
    use entity::{customer_address_entity, customer_entity};
    use std::sync::Arc;
    use uuid::Uuid;

    struct UnusedShippingProvider;
    #[business::sea_orm::prelude::async_trait::async_trait]
    impl ShippingProviderGateway for UnusedShippingProvider {
        async fn quote(
            &self,
            _request: ShippingRequest,
        ) -> Result<Vec<ShippingOption>, ShippingProviderError> {
            Err(ShippingProviderError::Unavailable)
        }
    }

    fn app_state(db: DbConn) -> AppState {
        let s3 = aws_sdk_s3::Client::from_conf(
            aws_sdk_s3::Config::builder()
                .behavior_version(aws_config::BehaviorVersion::latest())
                .build(),
        );
        AppState {
            conn: Arc::new(db),
            storage: Arc::new(business::gateway::storage_gateway::StorageGateway::new(
                "unused-test-bucket".into(),
                "http://localhost.invalid".into(),
                s3,
            )),
            shipping: Arc::new(UnusedShippingProvider),
            shipping_keys: Arc::new(crate::infrastructure::shipping_credentials::ShippingKeyRing::default()),
            payment_keys: Arc::new(crate::infrastructure::payment_credentials::PaymentKeyRing::default()),
            gateway_token: None,
        }
    }

    fn customer_user(id: i64) -> User {
        User {
            id: Some(id),
            uuid: Some(Uuid::new_v4().to_string()),
            email: format!("customer{id}@example.test"),
            name: Some("Customer".into()),
            password: String::new(),
            enabled: true,
            first_login: false,
            tenant_id: None,
            role: Role::Customer,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }

    fn customer_model(id: i64, user_id: i64, cpf: Option<&str>) -> customer_entity::Model {
        let now = Utc::now().naive_utc();
        customer_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            tenant_id: None,
            user_id: Some(user_id),
            name: "Customer".into(),
            email: format!("customer{user_id}@example.test"),
            cpf: cpf.map(str::to_owned),
            phone: Some("11999999999".into()),
            marketing_consent: false,
            consent_at: None,
            active: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    fn address_model(id: i64, customer_id: i64) -> customer_address_entity::Model {
        let now = Utc::now().naive_utc();
        customer_address_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            tenant_id: None,
            customer_id,
            label: Some("Home".into()),
            recipient: "Customer".into(),
            address_line1: Some("Street 1".into()),
            address_line2: None,
            locality: Some("City".into()),
            administrative_area: Some("SP".into()),
            postal_code: Some("01001000".into()),
            country_code: Some("BR".into()),
            is_default: true,
            created_at: now,
            created_by: None,
            updated_at: now,
            updated_by: None,
        }
    }

    #[test]
    fn cpf_check_digits() {
        assert!(valid_cpf("52998224725"));
        assert!(!valid_cpf("52998224726"));
        assert!(!valid_cpf("11111111111"));
    }

    #[tokio::test]
    async fn me_returns_profile_and_addresses_for_authenticated_user() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![customer_model(30, 7, Some("52998224725"))]])
            .append_query_results([vec![address_model(90, 30)]])
            .into_connection();
        let state = app_state(db);

        let response = me(
            State(state),
            Extension(customer_user(7)),
            Extension(Locale::En),
        )
        .await
        .unwrap();
        let Json(profile) = response;

        assert_eq!(profile.id, 30);
        assert_eq!(profile.email, "customer7@example.test");
        assert_eq!(profile.addresses.len(), 1);
        assert_eq!(profile.addresses[0].id, 90);
    }

    #[tokio::test]
    async fn me_refuses_authenticated_user_without_customer_record() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<customer_entity::Model>::new()])
            .into_connection();
        let state = app_state(db);

        let response = me(
            State(state),
            Extension(customer_user(99)),
            Extension(Locale::En),
        )
        .await;

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn cpf_completion_refuses_to_replace_existing_value() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![customer_model(30, 7, Some("52998224725"))]])
            .into_connection();
        let state = app_state(db);

        let response = complete_tax_id(
            State(state),
            Extension(customer_user(7)),
            Extension(Locale::En),
            Json(TaxIdInput {
                cpf: "11144477735".into(),
            }),
        )
        .await;

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn cpf_completion_rejects_invalid_and_duplicate_values() {
        let no_cpf = customer_model(30, 7, None);
        let invalid_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![no_cpf.clone()]])
            .into_connection();
        let invalid_response = complete_tax_id(
            State(app_state(invalid_db)),
            Extension(customer_user(7)),
            Extension(Locale::En),
            Json(TaxIdInput {
                cpf: "52998224726".into(),
            }),
        )
        .await;
        assert!(invalid_response.is_err());

        let duplicate = customer_model(31, 8, Some("52998224725"));
        let duplicate_db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![no_cpf]])
            .append_query_results([vec![duplicate]])
            .into_connection();
        let duplicate_response = complete_tax_id(
            State(app_state(duplicate_db)),
            Extension(customer_user(7)),
            Extension(Locale::En),
            Json(TaxIdInput {
                cpf: "52998224725".into(),
            }),
        )
        .await;
        assert!(duplicate_response.is_err());
    }
}
