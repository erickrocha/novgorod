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
    use super::valid_cpf;
    #[test]
    fn cpf_check_digits() {
        assert!(valid_cpf("52998224725"));
        assert!(!valid_cpf("52998224726"));
        assert!(!valid_cpf("11111111111"));
    }
}
