use crate::AppState;
use crate::commons::{exception_response::{ExceptionResponse, HttpResponse}, i18n::{ErrorKey, Locale}, pagination::{NormalizedPagination, PagedResponse}};
use crate::endpoints::json::{orders_json::*, error_response_json::ErrorResponseJson};
use crate::infrastructure::purchase_mapper::PurchaseMapper;
use axum::{Json, extract::{State, Extension, Path, Query}, http::{HeaderMap, StatusCode}};
use business::{domain::{user::User, marketplace::{PurchaseError, OrderFilter}}, gateway::purchase_gateway::PurchaseGateway, use_cases::purchase_use_case::PurchaseUseCase};

fn use_case(state: &AppState) -> PurchaseUseCase {
 PurchaseUseCase::new(PurchaseGateway::new(state.conn.as_ref().clone()))
}
fn error(locale: Locale, error: PurchaseError) -> ExceptionResponse {
 match error {
 PurchaseError::Validation(_) => ExceptionResponse::BadRequest(locale, ErrorKey::PurchaseInvalid),
 PurchaseError::NotFound => ExceptionResponse::NotFound(locale, ErrorKey::PurchaseNotFound),
 PurchaseError::Forbidden => ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden),
 PurchaseError::Conflict => ExceptionResponse::Conflict(locale, ErrorKey::PurchaseConflict),
 PurchaseError::Persistence(err) => {
 log::error!("Purchase persistence failed: {err}");
 ExceptionResponse::InternalServerError(locale, ErrorKey::PurchaseUnavailable)
 }
 }
}
fn filter(params: OrdersPageQuery, purchase: bool) -> (NormalizedPagination, OrderFilter) {
 let allowed = if purchase { &["id", "status", "totalCents", "createdAt"][..] } else {
 &["id", "number", "status", "paymentStatus", "totalCents", "placedAt", "createdAt"][..] };
 let mut norm = NormalizedPagination::new(&params.to_page_query(), allowed, "id");
 // Keep offsets within PostgreSQL's signed bigint range.
 norm.offset = norm.page.saturating_sub(1).saturating_mul(norm.page_size).min(i64::MAX as u64);
 let filter = OrderFilter { offset: norm.offset, limit: norm.page_size, query: norm.q.clone(),
 status: params.status, tenant_id: params.tenant_id, customer_id: params.customer_id,
 sort_by: norm.sort_by.clone(), descending: norm.sort_dir.is_descending() };
 (norm, filter)
}

#[utoipa::path(post, path = "/purchases", tag = "Purchases",
 params(("Idempotency-Key" = String, Header, description = "Required, 1–128 visible ASCII characters; reuse only for an equivalent request")),
 request_body(content = CreatePurchaseInputJson, example = json!({
 "items": [{"skuId": 101, "quantity": 3}, {"skuId": 202, "quantity": 2}],
 "shippingAddress": {"recipient": "Buyer", "addressLine1": "Rua Um, 10", "locality": "São Paulo", "administrativeArea": "SP", "postalCode": "01001000", "countryCode": "BR"}
 })),
 description = "Creates one pending purchase and one order per SKU seller. Catalog prices only; shipping, discounts and taxes are zero. Does not reserve inventory or charge a card. Only registered customers may create purchases.",
 responses((status = 201, description = "Created", body = PurchaseDetailJson),
 (status = 200, description = "Idempotent replay", body = PurchaseDetailJson),
 (status = 409, description = "Key reused with different request", body = ErrorResponseJson), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)),
 security(("bearer_auth" = [])))]
pub async fn create_purchase(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, headers: HeaderMap,
 input: Result<Json<CreatePurchaseInputJson>, axum::extract::rejection::JsonRejection>) -> HttpResponse<(StatusCode, Json<PurchaseDetailJson>)> {
 let Json(input) = input.map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::PurchaseInvalid))?;
 let key = headers.get("Idempotency-Key").and_then(|v| v.to_str().ok())
 .ok_or(ExceptionResponse::BadRequest(locale, ErrorKey::RequiredHeaderValueMissing))?;
 let created = use_case(&state).create(&user, key, PurchaseMapper::input(input)).await.map_err(|e| error(locale, e))?;
 Ok((if created.replayed { StatusCode::OK } else { StatusCode::CREATED }, Json(PurchaseMapper::json(created.detail))))
}

#[utoipa::path(get, path = "/purchases/{id}", tag = "Purchases", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = PurchaseDetailJson), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn get_purchase(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, Path(id): Path<i64>) -> HttpResponse<Json<PurchaseDetailJson>> {
 Ok(Json(PurchaseMapper::json(use_case(&state).get(&user, id).await.map_err(|e| error(locale, e))?)))
}

#[utoipa::path(get, path = "/orders/{id}", tag = "Orders", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = OrderDetailJson), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn get_by_id(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, Path(id): Path<i64>) -> HttpResponse<Json<OrderDetailJson>> {
 Ok(Json(use_case(&state).order(&user, id).await.map_err(|e| error(locale, e))?.into()))
}

#[utoipa::path(get, path = "/purchases/{id}/payments", tag = "Payments", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = Vec<PaymentDetailJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn payments(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, Path(id): Path<i64>) -> HttpResponse<Json<Vec<PaymentDetailJson>>> {
 Ok(Json(use_case(&state).get(&user, id).await.map_err(|e| error(locale, e))?.payments.into_iter().map(Into::into).collect()))
}

#[utoipa::path(get, path = "/orders/{id}/status-history", tag = "Orders", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = Vec<OrderStatusHistoryJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn history(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, Path(id): Path<i64>) -> HttpResponse<Json<Vec<OrderStatusHistoryJson>>> {
 Ok(Json(use_case(&state).history(&user, id).await.map_err(|e| error(locale, e))?.into_iter().map(Into::into).collect()))
}

#[utoipa::path(get, path = "/payments/{id}/transactions", tag = "Payments", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = Vec<PaymentTransactionJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn transactions(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, Path(id): Path<i64>) -> HttpResponse<Json<Vec<PaymentTransactionJson>>> {
 Ok(Json(use_case(&state).transactions(&user, id).await.map_err(|e| error(locale, e))?.into_iter().map(Into::into).collect()))
}

#[utoipa::path(get, path = "/orders/paged", tag = "Orders", params(OrdersPageQuery),
 responses((status = 200, description = "Accessible records", body = PagedResponse<OrdersJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn paged(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, Query(params): Query<OrdersPageQuery>) -> HttpResponse<Json<PagedResponse<OrdersJson>>> {
 let (norm, filter) = filter(params, false);
 let page = use_case(&state).orders(&user, &filter).await.map_err(|e| error(locale, e))?;
 Ok(Json(PagedResponse::new(page.items.into_iter().map(Into::into).collect(), page.total, norm.page, norm.page_size)))
}

#[utoipa::path(get, path = "/purchases/paged", tag = "Purchases", params(OrdersPageQuery),
 responses((status = 200, description = "Accessible records", body = PagedResponse<PurchaseJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn purchases_paged(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>, Query(params): Query<OrdersPageQuery>) -> HttpResponse<Json<PagedResponse<PurchaseJson>>> {
 let (norm, filter) = filter(params, true);
 let page = use_case(&state).purchases(&user, &filter).await.map_err(|e| error(locale, e))?;
 Ok(Json(PagedResponse::new(page.items.into_iter().map(Into::into).collect(), page.total, norm.page, norm.page_size)))
}

#[utoipa::path(get, path = "/orders", tag = "Orders",
 responses((status = 200, description = "Accessible orders; prefer /orders/paged for large result sets", body = Vec<OrdersJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn list_all(State(state): State<AppState>, Extension(user): Extension<User>,
 Extension(locale): Extension<Locale>) -> HttpResponse<Json<Vec<OrdersJson>>> {
 let filter = OrderFilter { offset: 0, limit: i64::MAX as u64, query: None, status: None,
 tenant_id: None, customer_id: None, sort_by: "id".into(), descending: true };
 let page = use_case(&state).orders(&user, &filter).await.map_err(|e| error(locale, e))?;
 Ok(Json(page.items.into_iter().map(Into::into).collect()))
}
