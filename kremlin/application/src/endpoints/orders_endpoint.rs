use crate::AppState;
use crate::commons::{
    tenant_context::TenantContext,
    exception_response::{ExceptionResponse, HttpResponse},
    i18n::{ErrorKey, Locale},
    pagination::{NormalizedPagination, PagedResponse},
};
use crate::endpoints::json::{error_response_json::ErrorResponseJson, orders_json::*};
use crate::infrastructure::purchase_mapper::PurchaseMapper;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use business::{
    domain::{
        marketplace::{OrderFilter, PurchaseError},
        user::User,
    },
    gateway::purchase_gateway::PurchaseGateway,
    use_cases::purchase_use_case::{CheckoutPurchaseInput, PurchaseUseCase},
};

fn use_case(state: &AppState) -> PurchaseUseCase {
    PurchaseUseCase::new(PurchaseGateway::new(state.conn.as_ref().clone()))
}
fn error(locale: Locale, error: PurchaseError) -> ExceptionResponse {
    match error {
        PurchaseError::DeliveryRateMissing(msg) => ExceptionResponse::CustomBadRequest(msg),
        PurchaseError::Validation(msg) => ExceptionResponse::CustomBadRequest(msg.into()),
        PurchaseError::NotFound => ExceptionResponse::NotFound(locale, ErrorKey::PurchaseNotFound),
        PurchaseError::Forbidden => {
            ExceptionResponse::Forbidden(locale, ErrorKey::PurchaseForbidden)
        }
        PurchaseError::Conflict => ExceptionResponse::Conflict(locale, ErrorKey::PurchaseConflict),
        PurchaseError::ShippingUnavailable => ExceptionResponse::ServiceUnavailable,
        PurchaseError::Persistence(err) => {
            log::error!("Purchase persistence failed: {err}");
            ExceptionResponse::InternalServerError(locale, ErrorKey::PurchaseUnavailable)
        }
    }
}
/// The resolved tenant is ANDed with the client's own `tenant_id` query parameter, never replaced
/// by it (SR-TEN-008). `Err(())` means the two differ, so nothing can match.
fn scope_tenant(context: TenantContext, query: Option<i64>) -> Result<Option<i64>, ()> {
    match (context.tenant_id(), query) {
        (Some(resolved), Some(asked)) if resolved != asked => Err(()),
        (Some(resolved), _) => Ok(Some(resolved)),
        (None, asked) => Ok(asked),
    }
}

fn empty_page<T>(norm: &NormalizedPagination) -> Json<PagedResponse<T>> {
    Json(PagedResponse::new(Vec::new(), 0, norm.page, norm.page_size))
}

fn filter(params: OrdersPageQuery, purchase: bool) -> (NormalizedPagination, OrderFilter) {
    let allowed = if purchase {
        &["id", "status", "totalCents", "createdAt"][..]
    } else {
        &[
            "id",
            "number",
            "status",
            "paymentStatus",
            "totalCents",
            "placedAt",
            "createdAt",
        ][..]
    };
    let mut norm = NormalizedPagination::new(&params.to_page_query(), allowed, "id");
    // Keep offsets within PostgreSQL's signed bigint range.
    norm.offset = norm
        .page
        .saturating_sub(1)
        .saturating_mul(norm.page_size)
        .min(i64::MAX as u64);
    let filter = OrderFilter {
        offset: norm.offset,
        limit: norm.page_size,
        query: norm.q.clone(),
        status: params.status,
        tenant_id: params.tenant_id,
        customer_id: params.customer_id,
        sort_by: norm.sort_by.clone(),
        descending: norm.sort_dir.is_descending(),
    };
    (norm, filter)
}

#[utoipa::path(post, path = "/purchases", tag = "Purchases",
 params(("Idempotency-Key" = String, Header, description = "Required, 1–128 visible ASCII characters; reuse only for an equivalent request")),
 request_body(content = CheckoutPurchaseJson, example = json!({"quoteId": 123, "email": "buyer@example.com", "phone": "11999999999"})),
 description = "Creates a purchase from an unexpired checkout quote and its selected shipping options. Rechecks catalog, stock, coupons and shipping inputs. Does not reserve inventory or charge a card. Idempotent retries retain the original quote and totals.",
 responses((status = 201, description = "Created", body = PurchaseDetailJson),
 (status = 200, description = "Idempotent replay", body = PurchaseDetailJson),
 (status = 409, description = "Key reused with different request", body = ErrorResponseJson), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)),
 security(("bearer_auth" = [])))]
pub async fn create_purchase(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Extension(context): Extension<TenantContext>,
    headers: HeaderMap,
    input: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>,
) -> HttpResponse<(StatusCode, Json<PurchaseDetailJson>)> {
    let Json(input) =
        input.map_err(|_| ExceptionResponse::BadRequest(locale, ErrorKey::PurchaseInvalid))?;
    let key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(ExceptionResponse::BadRequest(
            locale,
            ErrorKey::RequiredHeaderValueMissing,
        ))?;
    let quoted: CheckoutPurchaseJson = serde_json::from_value(input).map_err(|_| {
        ExceptionResponse::CustomBadRequest("valid checkout shipping quote required".into())
    })?;
    crate::endpoints::checkout_quote_endpoint::ensure_quote_tenant(&state, quoted.quote_id, context)
        .await?;
    let created = use_case(&state)
        .create_checkout(
            &user,
            key,
            CheckoutPurchaseInput {
                quote_id: quoted.quote_id,
                email: quoted.email,
                phone: quoted.phone,
            },
        )
        .await
        .map_err(|e| error(locale, e))?;
    Ok((
        if created.replayed {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(PurchaseMapper::json(created.detail)),
    ))
}

#[utoipa::path(get, path = "/purchases/{id}", tag = "Purchases", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = PurchaseDetailJson), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn get_purchase(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<PurchaseDetailJson>> {
    Ok(Json(PurchaseMapper::json(
        use_case(&state)
            .get(&user, id)
            .await
            .map_err(|e| error(locale, e))?,
    )))
}

#[utoipa::path(get, path = "/orders/{id}", tag = "Orders", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = OrderDetailJson), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn get_by_id(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Extension(context): Extension<TenantContext>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<OrderDetailJson>> {
    let detail = use_case(&state)
        .order(&user, id)
        .await
        .map_err(|e| error(locale, e))?;
    // an order of another tenant is "not found or not accessible" (SR-TEN-008)
    if context.tenant_id().is_some_and(|t| t != detail.order.tenant_id) {
        return Err(error(locale, PurchaseError::NotFound));
    }
    Ok(Json(detail.into()))
}

#[utoipa::path(get, path = "/purchases/{id}/payments", tag = "Payments", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = Vec<PaymentDetailJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn payments(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<Vec<PaymentDetailJson>>> {
    Ok(Json(
        use_case(&state)
            .get(&user, id)
            .await
            .map_err(|e| error(locale, e))?
            .payments
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "/orders/{id}/status-history", tag = "Orders", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = Vec<OrderStatusHistoryJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn history(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Extension(context): Extension<TenantContext>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<Vec<OrderStatusHistoryJson>>> {
    if let Some(tenant) = context.tenant_id() {
        let detail = use_case(&state)
            .order(&user, id)
            .await
            .map_err(|e| error(locale, e))?;
        if detail.order.tenant_id != tenant {
            return Err(error(locale, PurchaseError::NotFound));
        }
    }
    Ok(Json(
        use_case(&state)
            .history(&user, id)
            .await
            .map_err(|e| error(locale, e))?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "/payments/{id}/transactions", tag = "Payments", params(("id" = i64, Path)),
 description = "Customers access only their own resources. Seller staff access only their seller orders and their histories. Purchase/payment resources are restricted to the purchase owner and SysAdmin.",
 responses((status = 200, description = "Resource", body = Vec<PaymentTransactionJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn transactions(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Path(id): Path<i64>,
) -> HttpResponse<Json<Vec<PaymentTransactionJson>>> {
    Ok(Json(
        use_case(&state)
            .transactions(&user, id)
            .await
            .map_err(|e| error(locale, e))?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(get, path = "/orders/paged", tag = "Orders", params(OrdersPageQuery),
 responses((status = 200, description = "Accessible records", body = PagedResponse<OrdersJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn paged(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Extension(context): Extension<TenantContext>,
    Query(params): Query<OrdersPageQuery>,
) -> HttpResponse<Json<PagedResponse<OrdersJson>>> {
    let asked = params.tenant_id;
    let (norm, mut filter) = filter(params, false);
    match scope_tenant(context, asked) {
        Ok(tenant) => filter.tenant_id = tenant,
        Err(()) => return Ok(empty_page(&norm)),
    }
    let page = use_case(&state)
        .orders(&user, &filter)
        .await
        .map_err(|e| error(locale, e))?;
    Ok(Json(PagedResponse::new(
        page.items.into_iter().map(Into::into).collect(),
        page.total,
        norm.page,
        norm.page_size,
    )))
}

#[utoipa::path(get, path = "/purchases/paged", tag = "Purchases", params(OrdersPageQuery),
 responses((status = 200, description = "Accessible records", body = PagedResponse<PurchaseJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn purchases_paged(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Query(params): Query<OrdersPageQuery>,
) -> HttpResponse<Json<PagedResponse<PurchaseJson>>> {
    let (norm, filter) = filter(params, true);
    let page = use_case(&state)
        .purchases(&user, &filter)
        .await
        .map_err(|e| error(locale, e))?;
    Ok(Json(PagedResponse::new(
        page.items.into_iter().map(Into::into).collect(),
        page.total,
        norm.page,
        norm.page_size,
    )))
}

#[utoipa::path(get, path = "/orders", tag = "Orders",
 responses((status = 200, description = "Accessible orders; prefer /orders/paged for large result sets", body = Vec<OrdersJson>), (status = 400, description = "Invalid request", body = ErrorResponseJson),
(status = 401, description = "Unauthorized", body = ErrorResponseJson),
(status = 403, description = "Forbidden", body = ErrorResponseJson),
(status = 404, description = "Not found or not accessible", body = ErrorResponseJson),
(status = 500, description = "Persistence failure", body = ErrorResponseJson)), security(("bearer_auth" = [])))]
pub async fn list_all(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(locale): Extension<Locale>,
    Extension(context): Extension<TenantContext>,
) -> HttpResponse<Json<Vec<OrdersJson>>> {
    let filter = OrderFilter {
        offset: 0,
        limit: i64::MAX as u64,
        query: None,
        status: None,
        tenant_id: context.tenant_id(),
        customer_id: None,
        sort_by: "id".into(),
        descending: true,
    };
    let page = use_case(&state)
        .orders(&user, &filter)
        .await
        .map_err(|e| error(locale, e))?;
    Ok(Json(page.items.into_iter().map(Into::into).collect()))
}

#[cfg(test)]
#[path = "../../tests/unit/endpoints/orders_endpoint_test.rs"]
mod tenant_scope_tests;
