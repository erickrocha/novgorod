use crate::commons::entity_mapper::EntityMapper;
use crate::commons::gateway::Gateway;
use crate::domain::enums::Role;
use crate::domain::person::Person;
use crate::domain::user::{User, UserEntityMapper};
use crate::gateway::person_gateway::PersonGateway;
use crate::gateway::user_gateway::UserGateway;
use chrono::Utc;
use sea_orm::DbConn;
use std::env;

pub struct UserUseCase {
    gateway: UserGateway,
    person_gateway: PersonGateway,
}

impl UserUseCase {
    pub fn new(gateway: UserGateway) -> Self {
        let person_gateway = PersonGateway::new(gateway.db().clone());
        Self {
            gateway,
            person_gateway,
        }
    }

    pub fn new_with_person_gateway(gateway: UserGateway, person_gateway: PersonGateway) -> Self {
        Self {
            gateway,
            person_gateway,
        }
    }

    pub async fn create(&self, user: User) -> Option<User> {
        log::info!(
            "[UserUseCase::create] Executing for user email: {}",
            user.email
        );

        if user.email.trim().is_empty() {
            log::error!("[UserUseCase::create] User email is required");
            return None;
        }
        if user.password.trim().is_empty() {
            log::error!("[UserUseCase::create] User password is required");
            return None;
        }

        let encrypted_password = match bcrypt::hash(&user.password, bcrypt::DEFAULT_COST) {
            Ok(p) => p,
            Err(e) => {
                log::error!("[UserUseCase::create] Password encryption error: {}", e);
                return None;
            }
        };

        let user_to_save = User {
            password: encrypted_password,
            ..user
        };

        let entity = match self.gateway.persist(user_to_save).await {
            Ok(e) => e,
            Err(e) => {
                log::error!("[UserUseCase::create] Failed to persist user: {}", e);
                return None;
            }
        };

        let saved_user = UserEntityMapper::from_active_model(entity);

        if let Some(person) = Self::build_person_for_user(&saved_user) {
            let user_id = person.user_id;
            if let Err(e) = self.person_gateway.persist(person).await {
                log::error!("[UserUseCase::create] Failed to create person for user {}: {}", user_id, e);
                return None;
            }
        }

        Some(saved_user)
    }

    pub async fn update(&self, id: i64, user: User) -> Option<User> {
        log::info!(
            "[UserUseCase::update] Executing for user id {}: {}",
            id,
            user.email
        );

        let existing = match self.find_by_id(id).await {
            Some(u) => u,
            None => {
                log::error!(
                    "[UserUseCase::update] Failed to find existing user with id {}",
                    id
                );
                return None;
            }
        };

        let password = if user.password.trim().is_empty() {
            existing.password
        } else {
            match bcrypt::hash(&user.password, bcrypt::DEFAULT_COST) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("[UserUseCase::update] Password encryption error: {}", e);
                    return None;
                }
            }
        };

        let updated_user = User {
            id: Some(id),
            uuid: existing.uuid,
            email: if user.email.trim().is_empty() {
                existing.email
            } else {
                user.email
            },
            name: user.name.or(existing.name),
            password,
            enabled: user.enabled,
            first_login: user.first_login,
            tenant_id: user.tenant_id,
            role: user.role,
            created_at: existing.created_at,
            created_by: existing.created_by,
            updated_at: None,
            updated_by: user.updated_by,
        };

        let entity = match self.gateway.persist(updated_user).await {
            Ok(e) => e,
            Err(e) => {
                log::error!("[UserUseCase::update] Failed to update user: {}", e);
                return None;
            }
        };

        Some(UserEntityMapper::from_active_model(entity))
    }

    pub async fn change_password(
        &self,
        id: i64,
        current_password: String,
        new_password: String,
    ) -> Option<User> {
        log::info!(
            "[UserUseCase::change_password] Executing for user id {}",
            id
        );

        if new_password.trim().is_empty() {
            log::error!("[UserUseCase::change_password] New password is required");
            return None;
        }

        let existing = self.find_by_id(id).await?;

        if !bcrypt::verify(&current_password, existing.password.as_str()).unwrap_or(false) {
            log::error!("[UserUseCase::change_password] Current password is incorrect");
            return None;
        }

        let encrypted_password = match bcrypt::hash(&new_password, bcrypt::DEFAULT_COST) {
            Ok(p) => p,
            Err(e) => {
                log::error!("[UserUseCase::change_password] Password encryption error: {}", e);
                return None;
            }
        };

        let updated_user = User {
            password: encrypted_password,
            updated_at: None,
            first_login: false,
            ..existing
        };

        let entity = match self.gateway.persist(updated_user).await {
            Ok(e) => e,
            Err(e) => {
                log::error!("[UserUseCase::change_password] Failed to update password: {}", e);
                return None;
            }
        };

        Some(UserEntityMapper::from_active_model(entity))
    }

    pub async fn find_by_id(&self, id: i64) -> Option<User> {
        log::info!("[UserUseCase::find_by_id] Executing for id: {}", id);

        match self.gateway.find_by_id(id).await {
            Ok(Some(model)) => Some(UserEntityMapper::from_model(model)),
            Ok(None) => {
                log::error!("[UserUseCase::find_by_id] User not found with id: {}", id);
                None
            }
            Err(e) => {
                log::error!("[UserUseCase::find_by_id] Database error: {}", e);
                None
            }
        }
    }

    pub async fn find_all(&self) -> Vec<User> {
        log::info!("[UserUseCase::find_all] Executing find_all users");

        match self.gateway.find_all().await {
            Ok(entities) => UserEntityMapper::from_models(entities),
            Err(e) => {
                log::error!("[UserUseCase::find_all] Database error: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn find_all_by_tenant_id(&self, tenant_id: i64) -> Vec<User> {
        log::info!(
            "[UserUseCase::find_all_by_tenant_id] Executing for tenant_id: {}",
            tenant_id
        );

        match self.gateway.find_all_by_tenant_id(tenant_id).await {
            Ok(entities) => UserEntityMapper::from_models(entities),
            Err(e) => {
                log::error!("[UserUseCase::find_all_by_tenant_id] Database error: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn persist(db: DbConn, user: User) -> Option<User> {
        log::info!(
            "[UserUseCase::persist] Executing persist for user: {}",
            user.email
        );

        if user.email.is_empty() || user.password.is_empty() {
            log::error!("[UserUseCase::persist] Email or password is empty");
            return None;
        }

        let user_with_password_encrypted = User {
            password: bcrypt::hash(&user.password, bcrypt::DEFAULT_COST).unwrap(),
            ..user
        };

        let user_gateway = UserGateway::new(db.clone());
        let entity = user_gateway.persist(user_with_password_encrypted).await;
        if entity.is_err() {
            let error = entity.err().unwrap();
            log::error!("[UserUseCase::persist] Error adding user: {}", error);
            return None;
        }
        let user = entity.unwrap();
        let saved_user = UserEntityMapper::from_active_model(user);

        if let Some(person) = Self::build_person_for_user(&saved_user) {
            let person_gateway = PersonGateway::new(db);
            let _ = person_gateway.persist(person).await;
        }

        Some(saved_user)
    }

    pub async fn find_by_email(db: &DbConn, email: String) -> Option<User> {
        log::info!(
            "[UserUseCase::find_by_email] Executing for email: {}",
            email
        );

        let user_result = UserGateway::find_by_email(db, email.clone()).await;
        match user_result {
            Ok(Some(model)) => Some(UserEntityMapper::from_model(model)),
            Ok(None) => {
                log::info!(
                    "[UserUseCase::find_by_email] No user found for email: {}",
                    email
                );
                None
            }
            Err(error) => {
                log::error!(
                    "[UserUseCase::find_by_email] Error finding user by email {}: {}",
                    email,
                    error
                );
                None
            }
        }
    }

    async fn find_sysadmin(db: &DbConn) -> Option<User> {
        match UserGateway::find_by_role(db, Role::SysAdmin.to_string()).await {
            Ok(Some(model)) => Some(UserEntityMapper::from_model(model)),
            Ok(None) => None,
            Err(error) => {
                log::error!(
                    "[UserUseCase::find_sysadmin] Error finding SysAdmin user: {}",
                    error
                );
                None
            }
        }
    }

    /// Seeds the SysAdmin user from SYSADMIN_EMAIL/SYSADMIN_PASSWORD on every boot.
    /// Looks the existing SysAdmin up by role (not just by email) so that changing
    /// SYSADMIN_EMAIL updates the same user instead of leaving a stale one behind.
    pub async fn seed_sysadmin(db: &DbConn) -> Option<User> {
        log::info!("[UserUseCase::seed_sysadmin] Executing SysAdmin seed process");

        let sysadmin_email =
            env::var("SYSADMIN_EMAIL").unwrap_or_else(|_| "admin@afrodite.com".to_string());
        let sysadmin_password =
            env::var("SYSADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

        if sysadmin_email.trim().is_empty() || sysadmin_password.trim().is_empty() {
            log::warn!(
                "[UserUseCase::seed_sysadmin] SYSADMIN_EMAIL or SYSADMIN_PASSWORD empty; skipping SysAdmin seeding."
            );
            return None;
        }

        if let Some(existing_sysadmin) = Self::find_sysadmin(db).await {
            if existing_sysadmin.email == sysadmin_email {
                log::info!(
                    "[UserUseCase::seed_sysadmin] SysAdmin user already exists with email: {}",
                    sysadmin_email
                );
                return Some(existing_sysadmin);
            }

            log::info!(
                "[UserUseCase::seed_sysadmin] Updating SysAdmin user {} -> {}",
                existing_sysadmin.email,
                sysadmin_email
            );

            let updated_sysadmin = User {
                id: existing_sysadmin.id,
                uuid: existing_sysadmin.uuid,
                email: sysadmin_email,
                name: existing_sysadmin.name,
                password: sysadmin_password,
                enabled: true,
                first_login: existing_sysadmin.first_login,
                tenant_id: existing_sysadmin.tenant_id,
                role: Role::SysAdmin,
                created_at: existing_sysadmin.created_at,
                created_by: Some("system".to_string()),
                updated_at: existing_sysadmin.created_at,
                updated_by: Some("system".to_string()),
            };

            let updated = Self::persist(db.clone(), updated_sysadmin).await;
            if updated.is_some() {
                log::info!("[UserUseCase::seed_sysadmin] SysAdmin user updated successfully.");
            } else {
                log::error!("[UserUseCase::seed_sysadmin] Failed to update SysAdmin user.");
            }
            return updated;
        }

        log::info!(
            "[UserUseCase::seed_sysadmin] Creating initial SysAdmin user with email: {}",
            sysadmin_email
        );

        let sysadmin_user = User {
            id: None,
            uuid: None,
            email: sysadmin_email,
            name: Some("System Administrator".to_string()),
            password: sysadmin_password,
            enabled: true,
            first_login: false,
            tenant_id: None,
            role: Role::SysAdmin,
            created_at: Some(Utc::now().naive_utc()),
            created_by: Some("system".to_string()),
            updated_at: Some(Utc::now().naive_utc()),
            updated_by: Some("system".to_string()),
        };

        let created = Self::persist(db.clone(), sysadmin_user).await;
        if created.is_some() {
            log::info!("[UserUseCase::seed_sysadmin] Initial SysAdmin user created successfully.");
        } else {
            log::error!("[UserUseCase::seed_sysadmin] Failed to create initial SysAdmin user.");
        }
        created
    }

    pub fn build_person_for_user(user: &User) -> Option<Person> {
        let user_id = user.id?;
        let first_name = match user.name.as_deref() {
            Some(name) if !name.trim().is_empty() => name.trim().to_string(),
            _ => user
                .email
                .split('@')
                .next()
                .unwrap_or("User")
                .to_string(),
        };

        Some(Person {
            id: None,
            uuid: None,
            tenant_id: user.tenant_id,
            user_id,
            first_name,
            surname: None,
            date_of_birth: None,
            gender: None,
            avatar: None,
            phone: None,
            email: None,
            created_at: None,
            created_by: user.created_by.clone(),
            updated_at: None,
            updated_by: None,
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/use_cases/user_use_case.rs"]
mod tests;
