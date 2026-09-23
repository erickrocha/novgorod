use crate::domain::business_error::BusinessError;
use crate::domain::customer::Customer;
use crate::domain::customer_address::CustomerAddress;
use crate::domain::enums::Role;
use crate::domain::user::User;
use crate::gateway::customer_address_gateway::CustomerAddressGateway;
use crate::gateway::customer_gateway::CustomerGateway;
use crate::gateway::user_gateway::UserGateway;
use crate::commons::gateway::Gateway;
use sea_orm::{DbConn, TransactionTrait};

pub struct CustomerRegistrationUseCase {}

impl CustomerRegistrationUseCase {
    pub async fn execute(
        db: &DbConn,
        mut user: User,
        mut customer: Customer,
        address: Option<CustomerAddress>,
    ) -> Result<Customer, BusinessError> {
        let txn = db.begin().await.map_err(|e| {
            log::error!("Failed to begin transaction: {}", e);
            BusinessError::new("Internal server error".to_string())
        })?;

        // 0. Validate if Email or CPF already exists
        if UserGateway::find_by_email(db, user.email.clone()).await.unwrap_or(None).is_some() {
            return Err(BusinessError::new("E-mail já está em uso. Por favor, recupere sua conta.".to_string()));
        }

        if let Some(cpf) = customer.cpf.as_ref()
            && let Some(_) = CustomerGateway::find_by_cpf(db, cpf.clone()).await.unwrap_or(None) {
                return Err(BusinessError::new("CPF já está em uso. Por favor, recupere sua conta.".to_string()));
            }

        // 1. Persist User
        user.role = Role::Customer;
        
        // hash the password
        user.password = bcrypt::hash(user.password.as_str(), bcrypt::DEFAULT_COST)
            .unwrap_or(user.password);

        let user_gateway = UserGateway::new(db.clone());
        let persisted_user = user_gateway.persist(user).await.map_err(|e| {
            log::error!("Failed to persist user: {}", e);
            BusinessError::new("Failed to create user".to_string())
        })?;
        
        let user_id = persisted_user.id.clone().unwrap();

        // 2. Persist Customer
        customer.user_id = Some(user_id);
        let customer_gateway = CustomerGateway::new(db.clone());
        let persisted_customer = customer_gateway.persist(customer).await.map_err(|e| {
            log::error!("Failed to persist customer: {}", e);
            BusinessError::new("Failed to create customer".to_string())
        })?;

        let customer_id = persisted_customer.id.clone().unwrap();

        // 3. Persist CustomerAddress if provided
        if let Some(mut addr) = address {
            addr.customer_id = customer_id;
            let address_gateway = CustomerAddressGateway::new(db.clone());
            address_gateway.persist(addr).await.map_err(|e| {
                log::error!("Failed to persist customer address: {}", e);
                BusinessError::new("Failed to create customer address".to_string())
            })?;
        }

        txn.commit().await.map_err(|e| {
            log::error!("Failed to commit transaction: {}", e);
            BusinessError::new("Internal server error".to_string())
        })?;

        use crate::domain::customer::CustomerEntityMapper;
        use crate::commons::entity_mapper::EntityMapper;
        Ok(CustomerEntityMapper::from_active_model(persisted_customer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use entity::{customer_address_entity, customer_entity, user_entity};
    use sea_orm::{DatabaseBackend, DbErr, Iden, MockDatabase};
    use uuid::Uuid;

    fn build_test_user(email: &str) -> User {
        User {
            id: None,
            uuid: None,
            email: email.to_string(),
            name: Some("Jane Doe".to_string()),
            password: "PlainPassword123".to_string(),
            enabled: true,
            first_login: false,
            tenant_id: Some(1),
            role: Role::Customer,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }

    fn build_test_customer(cpf: Option<&str>) -> Customer {
        Customer {
            id: None,
            uuid: None,
            tenant_id: Some(1),
            user_id: None,
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            cpf: cpf.map(|s| s.to_string()),
            phone: Some("11987654321".to_string()),
            marketing_consent: true,
            consent_at: None,
            active: true,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
        }
    }

    fn build_test_address() -> CustomerAddress {
        CustomerAddress {
            id: None,
            uuid: None,
            tenant_id: Some(1),
            customer_id: 0,
            label: Some("Casa".to_string()),
            recipient: "Jane Doe".to_string(),
            postal_code: Some("01001-000".to_string()),
            address_line1: Some("Praça da Sé".to_string()),
            address_line2: Some("100".to_string()),
            locality: Some("Florianopolis".to_string()),
            country_code: Some("BR".to_string()),
            is_default: true,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            administrative_area: None,
        }
    }

    fn mock_user_model(id: i64, email: &str) -> user_entity::Model {
        user_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            name: Some("Jane Doe".to_string()),
            email: email.to_string(),
            password: "hashed_password".to_string(),
            first_login: false,
            enabled: true,
            tenant_id: Some(1),
            role: "customer".to_string(),
            blocked_reason: None,
            created_at: Option::from(Utc::now().naive_utc()),
            created_by: None,
            updated_at: Option::from(Utc::now().naive_utc()),
            updated_by: None,
        }
    }

    fn mock_customer_model(id: i64, user_id: Option<i64>, cpf: Option<&str>) -> customer_entity::Model {
        customer_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            tenant_id: Some(1),
            user_id,
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            cpf: cpf.map(|s| s.to_string()),
            phone: Some("11987654321".to_string()),
            marketing_consent: true,
            consent_at: Some(Utc::now().naive_utc()),
            active: true,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None,
        }
    }

    fn mock_address_model(id: i64, customer_id: i64) -> customer_address_entity::Model {
        customer_address_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            tenant_id: Some(1),
            customer_id,
            label: Some("Casa".to_string()),
            recipient: "Jane Doe".to_string(),
            postal_code: Some("01001-000".to_string()),
            address_line1: Some("Praça da Sé".to_string()),
            address_line2: Some("100".to_string()),
            locality: Some("Florianopolis".to_string()),
            country_code: Some("BR".to_string()),
            is_default: true,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None,
            administrative_area: Some("Santa Catarina".to_string()),
        }
    }

    #[tokio::test]
    async fn test_registration_success_without_address() {
        let user = build_test_user("jane@example.com");
        let customer = build_test_customer(Some("12345678901"));

        let user_res = mock_user_model(1, "jane@example.com");
        let customer_res = mock_customer_model(10, Some(1), Some("12345678901"));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()]) // find_by_email -> None
            .append_query_results([Vec::<customer_entity::Model>::new()]) // find_by_cpf -> None
            .append_query_results([vec![user_res]]) // user persist save
            .append_query_results([vec![customer_res]]) // customer persist save
            .into_connection();

        let result = CustomerRegistrationUseCase::execute(&db, user, customer, None)
            .await
            .expect("registration without address should succeed");

        assert_eq!(result.id, Some(10));
        assert_eq!(result.user_id, Some(1));
    }

    #[tokio::test]
    async fn test_registration_success_with_address() {
        let user = build_test_user("jane@example.com");
        let customer = build_test_customer(Some("12345678901"));
        let address = build_test_address();

        let user_res = mock_user_model(2, "jane@example.com");
        let customer_res = mock_customer_model(20, Some(2), Some("12345678901"));
        let address_res = mock_address_model(100, 20);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()]) // find_by_email -> None
            .append_query_results([Vec::<customer_entity::Model>::new()]) // find_by_cpf -> None
            .append_query_results([vec![user_res]]) // user persist
            .append_query_results([vec![customer_res]]) // customer persist
            .append_query_results([vec![address_res]]) // address persist
            .into_connection();

        let result = CustomerRegistrationUseCase::execute(&db, user, customer, Some(address))
            .await
            .expect("registration with address should succeed");

        assert_eq!(result.id, Some(20));
        assert_eq!(result.user_id, Some(2));
    }

    #[tokio::test]
    async fn test_registration_success_without_cpf() {
        let user = build_test_user("nocpf@example.com");
        let customer = build_test_customer(None);

        let user_res = mock_user_model(3, "nocpf@example.com");
        let customer_res = mock_customer_model(30, Some(3), None);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()]) // find_by_email -> None
            .append_query_results([vec![user_res]]) // user persist
            .append_query_results([vec![customer_res]]) // customer persist
            .into_connection();

        let result = CustomerRegistrationUseCase::execute(&db, user, customer, None)
            .await
            .expect("registration without cpf should succeed");

        assert_eq!(result.id, Some(30));
        assert_eq!(result.user_id, Some(3));
    }

    #[tokio::test]
    async fn test_registration_duplicate_email() {
        let user = build_test_user("existing@example.com");
        let customer = build_test_customer(Some("12345678901"));

        let existing_user = mock_user_model(99, "existing@example.com");

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![existing_user]]) // find_by_email -> Some
            .into_connection();

        let err = CustomerRegistrationUseCase::execute(&db, user, customer, None)
            .await
            .unwrap_err();

        assert_eq!(err.message, "E-mail já está em uso. Por favor, recupere sua conta.");
    }

    #[tokio::test]
    async fn test_registration_duplicate_cpf() {
        let user = build_test_user("new@example.com");
        let customer = build_test_customer(Some("99999999999"));

        let existing_customer = mock_customer_model(88, Some(88), Some("99999999999"));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()]) // find_by_email -> None
            .append_query_results([vec![existing_customer]]) // find_by_cpf -> Some
            .into_connection();

        let err = CustomerRegistrationUseCase::execute(&db, user, customer, None)
            .await
            .unwrap_err();

        assert_eq!(err.message, "CPF já está em uso. Por favor, recupere sua conta.");
    }

    #[tokio::test]
    async fn test_registration_user_persist_failure() {
        let user = build_test_user("jane@example.com");
        let customer = build_test_customer(Some("12345678901"));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()]) // find_by_email -> None
            .append_query_results([Vec::<customer_entity::Model>::new()]) // find_by_cpf -> None
            .append_query_errors([DbErr::Custom("Insert user failed".to_string())])
            .into_connection();

        let err = CustomerRegistrationUseCase::execute(&db, user, customer, None)
            .await
            .unwrap_err();

        assert_eq!(err.message, "Failed to create user");
    }

    #[tokio::test]
    async fn test_registration_customer_persist_failure() {
        let user = build_test_user("jane@example.com");
        let customer = build_test_customer(Some("12345678901"));
        let user_res = mock_user_model(1, "jane@example.com");

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()]) // find_by_email -> None
            .append_query_results([Vec::<customer_entity::Model>::new()]) // find_by_cpf -> None
            .append_query_results([vec![user_res]]) // user persist ok
            .append_query_errors([DbErr::Custom("Insert customer failed".to_string())])
            .into_connection();

        let err = CustomerRegistrationUseCase::execute(&db, user, customer, None)
            .await
            .unwrap_err();

        assert_eq!(err.message, "Failed to create customer");
    }

    #[tokio::test]
    async fn test_registration_address_persist_failure() {
        let user = build_test_user("jane@example.com");
        let customer = build_test_customer(Some("12345678901"));
        let address = build_test_address();

        let user_res = mock_user_model(1, "jane@example.com");
        let customer_res = mock_customer_model(10, Some(1), Some("12345678901"));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()]) // find_by_email -> None
            .append_query_results([Vec::<customer_entity::Model>::new()]) // find_by_cpf -> None
            .append_query_results([vec![user_res]]) // user persist ok
            .append_query_results([vec![customer_res]]) // customer persist ok
            .append_query_errors([DbErr::Custom("Insert address failed".to_string())])
            .into_connection();

        let err = CustomerRegistrationUseCase::execute(&db, user, customer, Some(address))
            .await
            .unwrap_err();

        assert_eq!(err.message, "Failed to create customer address");
    }
}
