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
    pub async fn execute(db: &DbConn,mut user: User,mut customer: Customer,address: Option<CustomerAddress>) -> Result<Customer, BusinessError> {
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
#[path = "../../tests/unit/use_cases/customer_registration_use_case.rs"]
mod tests;
