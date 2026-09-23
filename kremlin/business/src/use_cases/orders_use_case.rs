use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::orders::{Orders, OrdersEntityMapper};
use crate::gateway::orders_gateway::OrdersGateway;

pub struct OrdersUseCase {
    gateway: OrdersGateway,
}

impl OrdersUseCase {
    pub fn new(gateway: OrdersGateway) -> Self {
        Self { gateway }
    }

    pub async fn create(&self, orders: Orders) -> Option<Orders> {
        let entity = self.gateway.persist(orders).await.map_err(|e| {
            log::error!("Failed to persist order: {}", e);
        }).ok()?;
        Some(OrdersEntityMapper::from_active_model(entity))
    }

    pub async fn find_all(&self) -> Vec<Orders> {
        let entities = self.gateway.find_all().await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).unwrap_or_default();
        OrdersEntityMapper::from_models(entities)
    }

    pub async fn find_by_id(&self, id: i64) -> Option<Orders> {
        let entity = self.gateway.find_by_id(id).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrdersEntityMapper::from_model(entity))
    }

    pub async fn find_by_uuid(&self, uuid: String) -> Option<Orders> {
        let entity = self.gateway.find_by_uuid(uuid).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrdersEntityMapper::from_model(entity))
    }

    pub async fn find_by_number(&self, number: String) -> Option<Orders> {
        let entity = self.gateway.find_by_number(number).await.map_err(|e| {
            log::error!("Database error: {}", e);
        }).ok()??;
        Some(OrdersEntityMapper::from_model(entity))
    }

    pub async fn find_by_customer_id(&self, customer_id: i64) -> Vec<Orders> {
        let entities = self
            .gateway
            .find_by_customer_id(customer_id)
            .await
            .map_err(|e| {
                log::error!("Database error: {}", e);
            })
            .unwrap_or_default();
        OrdersEntityMapper::from_models(entities)
    }

    pub async fn update(&self, id: i64, mut orders: Orders) -> Option<Orders> {
        orders.id = Some(id);
        let entity = self.gateway.persist(orders).await.map_err(|e| {
            log::error!("Failed to update order: {}", e);
        }).ok()?;
        Some(OrdersEntityMapper::from_active_model(entity))
    }

    pub async fn delete_by_id(&self, id: i64) -> Option<()> {
        self.gateway.delete_by_id(id).await.map_err(|e| {
            log::error!("Failed to delete order: {}", e);
        }).ok()?;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use entity::orders_entity;
    use sea_orm::{DatabaseBackend, DbErr, MockDatabase, MockExecResult};
    use uuid::Uuid;

    fn build_test_order() -> Orders {
        Orders {
            id: None,
            uuid: None,
            tenant_id: Some(1),
            number: "ORD-001".to_string(),
            customer_id: 10,
            status: "PENDING".to_string(),
            payment_status: "UNPAID".to_string(),
            subtotal_cents: 10000,
            discount_cents: 0,
            shipping_cents: 1500,
            tax_total_cents: 500,
            total_cents: 12000,
            coupon_id: None,
            coupon_code: None,
            ship_recipient: "John Buyer".to_string(),
            ship_cep: "01001-000".to_string(),
            ship_logradouro: "Av Paulista".to_string(),
            ship_numero: "1000".to_string(),
            ship_complemento: None,
            ship_bairro: "Bela Vista".to_string(),
            ship_cidade: "São Paulo".to_string(),
            ship_uf: "SP".to_string(),
            placed_at: Utc::now().naive_utc(),
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }

    fn mock_order_model(id: i64, number: &str, customer_id: i64) -> orders_entity::Model {
        orders_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            tenant_id: Some(1),
            number: number.to_string(),
            customer_id,
            status: "PENDING".to_string(),
            payment_status: "UNPAID".to_string(),
            subtotal_cents: 10000,
            discount_cents: 0,
            shipping_cents: 1500,
            tax_total_cents: 500,
            total_cents: 12000,
            coupon_id: None,
            coupon_code: None,
            ship_recipient: "John Buyer".to_string(),
            ship_cep: "01001-000".to_string(),
            ship_logradouro: "Av Paulista".to_string(),
            ship_numero: "1000".to_string(),
            ship_complemento: None,
            ship_bairro: "Bela Vista".to_string(),
            ship_cidade: "São Paulo".to_string(),
            ship_uf: "SP".to_string(),
            placed_at: Utc::now().naive_utc(),
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None,
        }
    }

    #[tokio::test]
    async fn test_create_order_success_and_failure() {
        let order = build_test_order();
        let model = mock_order_model(1, "ORD-001", 10);

        // Success
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model]])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_ok));
        let created = use_case.create(order.clone()).await.expect("order created");
        assert_eq!(created.id, Some(1));
        assert_eq!(created.number, "ORD-001");

        // Failure
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Insert failed".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        assert!(use_case_err.create(order).await.is_none());
    }

    #[tokio::test]
    async fn test_find_all_orders_success_and_failure() {
        let model1 = mock_order_model(1, "ORD-001", 10);
        let model2 = mock_order_model(2, "ORD-002", 11);

        // Success
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model1, model2]])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_ok));
        let all = use_case.find_all().await;
        assert_eq!(all.len(), 2);

        // Failure
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Select failed".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        let empty = use_case_err.find_all().await;
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_find_by_id_found_not_found_error() {
        let model = mock_order_model(42, "ORD-042", 10);

        // Found
        let db_found = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model]])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_found));
        let found = use_case.find_by_id(42).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().number, "ORD-042");

        // Not found
        let db_not_found = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<orders_entity::Model>::new()])
            .into_connection();
        let use_case_nf = OrdersUseCase::new(OrdersGateway::new(db_not_found));
        assert!(use_case_nf.find_by_id(999).await.is_none());

        // DB error
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("DB timeout".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        assert!(use_case_err.find_by_id(42).await.is_none());
    }

    #[tokio::test]
    async fn test_find_by_uuid_found_not_found_error() {
        let model = mock_order_model(7, "ORD-007", 10);
        let uuid_str = model.uuid.to_string();

        // Found
        let db_found = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model]])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_found));
        let found = use_case.find_by_uuid(uuid_str.clone()).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, Some(7));

        // Not found
        let db_not_found = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<orders_entity::Model>::new()])
            .into_connection();
        let use_case_nf = OrdersUseCase::new(OrdersGateway::new(db_not_found));
        assert!(use_case_nf.find_by_uuid(Uuid::new_v4().to_string()).await.is_none());

        // Error
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("UUID query failed".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        assert!(use_case_err.find_by_uuid(uuid_str).await.is_none());
    }

    #[tokio::test]
    async fn test_find_by_number_found_not_found_error() {
        let model = mock_order_model(8, "ORD-008", 10);

        // Found
        let db_found = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model]])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_found));
        let found = use_case.find_by_number("ORD-008".to_string()).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, Some(8));

        // Not found
        let db_not_found = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<orders_entity::Model>::new()])
            .into_connection();
        let use_case_nf = OrdersUseCase::new(OrdersGateway::new(db_not_found));
        assert!(use_case_nf.find_by_number("NONEXISTENT".to_string()).await.is_none());

        // DB error
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Query failed".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        assert!(use_case_err.find_by_number("ORD-008".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_find_by_customer_id_success_and_error() {
        let model1 = mock_order_model(1, "ORD-001", 100);
        let model2 = mock_order_model(2, "ORD-002", 100);

        // Success
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![model1, model2]])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_ok));
        let customer_orders = use_case.find_by_customer_id(100).await;
        assert_eq!(customer_orders.len(), 2);

        // DB error returns empty vector
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Customer query error".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        let empty = use_case_err.find_by_customer_id(100).await;
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_update_order_success_and_failure() {
        let order = build_test_order();
        let updated_model = mock_order_model(5, "ORD-005-UPDATED", 10);

        // Success
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![updated_model]])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_ok));
        let updated = use_case.update(5, order.clone()).await;
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().number, "ORD-005-UPDATED");

        // Failure
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Update failed".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        assert!(use_case_err.update(5, order).await.is_none());
    }

    #[tokio::test]
    async fn test_delete_by_id_success_and_failure() {
        // Success
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let use_case = OrdersUseCase::new(OrdersGateway::new(db_ok));
        assert!(use_case.delete_by_id(10).await.is_some());

        // Failure
        let db_err = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_errors([DbErr::Custom("Delete failed".to_string())])
            .into_connection();
        let use_case_err = OrdersUseCase::new(OrdersGateway::new(db_err));
        assert!(use_case_err.delete_by_id(10).await.is_none());
    }
}
