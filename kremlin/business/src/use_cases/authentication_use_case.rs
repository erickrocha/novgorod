use crate::commons::entity_mapper::EntityMapper;
use crate::domain::access_token::{AccessToken, Claims};
use crate::domain::business_error::BusinessError;
use crate::domain::user::{User, UserEntityMapper};
use crate::gateway::customer_gateway::CustomerGateway;
use crate::gateway::user_gateway::UserGateway;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::DbConn;
use std::env;

pub struct AuthenticationUseCase {}

impl AuthenticationUseCase {
    /// Carrega o usuário pelo e-mail e recusa conta desabilitada.
    ///
    /// Todo caminho de autenticação passa por aqui — login, validação de access
    /// token e de refresh token —, então é este o ponto que faz `user.enabled`
    /// valer. Sem ele o flag existe no schema e não protege nada: desabilitar
    /// uma conta (ou atender um pedido de exclusão) não derrubaria a sessão em
    /// curso nem impediria um novo login.
    async fn load_enabled_user(
        db: &DbConn,
        email: &str,
        context: &str,
    ) -> Result<User, BusinessError> {
        let found = UserGateway::find_by_email(db, email.to_string())
            .await
            .map_err(|err| {
                log::error!(
                    "[AuthenticationUseCase::{}] Database error for user {}: {}",
                    context,
                    email,
                    err
                );
                BusinessError::new("Invalid credentials".to_string())
            })?;

        let Some(model) = found else {
            log::error!(
                "[AuthenticationUseCase::{}] User not found: {}",
                context,
                email
            );
            return Err(BusinessError::new("Invalid credentials".to_string()));
        };

        if !model.enabled {
            log::warn!(
                "[AuthenticationUseCase::{}] Disabled account rejected: {}",
                context,
                email
            );
            return Err(BusinessError::new("Invalid credentials".to_string()));
        }

        Ok(UserEntityMapper::from_model(model))
    }

    pub async fn execute(
        db: &DbConn,
        email: String,
        password: String,
    ) -> Result<AccessToken, BusinessError> {
        log::info!(
            "[AuthenticationUseCase::execute] Executing login for user: {}",
            email
        );
        if email.is_empty() || password.is_empty() {
            log::error!("[AuthenticationUseCase::execute] Email and password are required");
            return Err(BusinessError::new(
                "Email and password are required".to_string(),
            ));
        }
        // Determine if it's an email or CPF
        let user = if email.contains('@') {
            Self::load_enabled_user(db, &email, "execute").await?
        } else {
            // It might be a CPF. Strip non-numeric characters.
            let cpf: String = email.chars().filter(|c| c.is_ascii_digit()).collect();
            if cpf.is_empty() {
                return Err(BusinessError::new("Invalid credentials".to_string()));
            }

            let customer = CustomerGateway::find_by_cpf(db, cpf).await.map_err(|err| {
                log::error!("[AuthenticationUseCase::execute] Database error looking up CPF: {}", err);
                BusinessError::new("Invalid credentials".to_string())
            })?;

            if let Some(c) = customer {
                if let Some(uid) = c.user_id {
                    let user_model = UserGateway::find_by_id_static(db, uid).await.map_err(|_| {
                        BusinessError::new("Invalid credentials".to_string())
                    })?;
                    
                    if let Some(um) = user_model {
                        if !um.enabled {
                            return Err(BusinessError::new("Invalid credentials".to_string()));
                        }
                        UserEntityMapper::from_model(um)
                    } else {
                        return Err(BusinessError::new("Invalid credentials".to_string()));
                    }
                } else {
                    return Err(BusinessError::new("Invalid credentials".to_string()));
                }
            } else {
                return Err(BusinessError::new("Invalid credentials".to_string()));
            }
        };

        if bcrypt::verify(password, user.password.as_str()).unwrap_or(false) {
            log::info!(
                "[AuthenticationUseCase::execute] Password verified for user: {}",
                email
            );
            let access_token = Self::generate_access_token(user);
            Ok(access_token)
        } else {
            log::error!(
                "[AuthenticationUseCase::execute] Invalid password for user: {}",
                email
            );
            Err(BusinessError::new("Invalid credentials".to_string()))
        }
    }

    pub fn generate_access_token(user: User) -> AccessToken {
        log::info!(
            "[AuthenticationUseCase::generate_access_token] Generating access token for: {}",
            user.email
        );
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(3))
            .expect("valid timestamp")
            .timestamp();
        let claims = Claims::builder()
            .with_sub(user.email.clone())
            .exp(expiration)
            .uuid(user.email.clone())
            .name(user.name.clone().unwrap_or_default())
            .user_id(user.id.unwrap_or(0))
            .role(user.role.clone())
            .tenant_id(user.tenant_id)
            .build()
            .expect("missing required claims field");

        let header = Header::new(Algorithm::HS512);
        let private_key = env::var("ACCESS_TOKEN_SECRET").expect("ACCESS_TOKEN_SECRET must be set");
        let token = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(private_key.as_bytes()),
        )
        .unwrap();
        let refresh_token = Self::generate_refresh_token(user.clone());
        AccessToken {
            access_token: token,
            token_type: "Bearer".to_string(),
            expire_in: expiration,
            refresh_token: Some(refresh_token),
            email: claims.sub.clone(),
            uuid: claims.uuid.clone(),
            name: claims.name.clone(),
            user_id: claims.user_id,
            role: claims.role,
            tenant_id: claims.tenant_id,
            first_login: user.first_login,
        }
    }

    fn generate_refresh_token(user: User) -> String {
        log::info!(
            "[AuthenticationUseCase::generate_refresh_token] Generating refresh token for: {}",
            user.email
        );
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::days(7))
            .expect("valid timestamp")
            .timestamp();
        let claims = Claims::builder()
            .with_sub(user.email.clone())
            .exp(expiration)
            .uuid(user.email.clone())
            .name(user.name.clone().unwrap_or_default())
            .user_id(user.id.unwrap_or(0))
            .role(user.role.clone())
            .tenant_id(user.tenant_id)
            .build()
            .expect("missing required claims field");

        let header = Header::new(Algorithm::HS512);
        let private_key =
            env::var("REFRESH_TOKEN_SECRET").expect("REFRESH_TOKEN_SECRET must be set");
        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(private_key.as_bytes()),
        )
        .unwrap()
    }

    pub async fn validate(db: &DbConn, token: String) -> Result<User, BusinessError> {
        log::info!("[AuthenticationUseCase::validate] Validating access token");
        let public_key = env::var("ACCESS_TOKEN_SECRET").expect("ACCESS_TOKEN_SECRET must be set");
        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(public_key.as_bytes()),
            &Validation::new(Algorithm::HS512),
        );

        if let Err(err) = &result {
            log::error!(
                "[AuthenticationUseCase::validate] Token decode error: {:?}",
                err
            );
            return Err(BusinessError::new("Token is invalid".to_string()));
        }

        let authentication = result.unwrap();
        log::info!(
            "[AuthenticationUseCase::validate] Token valid for subject: {}",
            authentication.claims.sub
        );
        let email = authentication.claims.sub;

        Self::load_enabled_user(db, &email, "validate").await
    }

    pub async fn validate_refresh_token(db: &DbConn, token: String) -> Result<User, BusinessError> {
        log::info!("[AuthenticationUseCase::validate_refresh_token] Validating refresh token");
        let public_key =
            env::var("REFRESH_TOKEN_SECRET").expect("REFRESH_TOKEN_SECRET must be set");
        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(public_key.as_bytes()),
            &Validation::new(Algorithm::HS512),
        );

        if let Err(err) = &result {
            log::error!(
                "[AuthenticationUseCase::validate_refresh_token] Refresh token decode error: {:?}",
                err
            );
            return Err(BusinessError::new("Token is invalid".to_string()));
        }

        let authentication = result.unwrap();
        log::info!(
            "[AuthenticationUseCase::validate_refresh_token] Refresh token valid for subject: {}",
            authentication.claims.sub
        );
        let email = authentication.claims.sub;

        Self::load_enabled_user(db, &email, "validate_refresh_token").await
    }

    pub async fn refresh_token(
        db: &DbConn,
        refresh_token: String,
    ) -> Result<AccessToken, BusinessError> {
        log::info!("[AuthenticationUseCase::refresh_token] Refreshing token");
        let user = AuthenticationUseCase::validate_refresh_token(db, refresh_token).await?;
        Ok(AuthenticationUseCase::generate_access_token(user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use entity::{customer_entity, user_entity};
    use sea_orm::{DatabaseBackend, DbErr, MockDatabase};
    use uuid::Uuid;

    fn setup_env() {
        unsafe {
            std::env::set_var(
                "ACCESS_TOKEN_SECRET",
                "test_access_secret_which_is_sufficiently_long_for_hs512_alg_safety_1234567890",
            );
            std::env::set_var(
                "REFRESH_TOKEN_SECRET",
                "test_refresh_secret_which_is_sufficiently_long_for_hs512_alg_safety_1234567890",
            );
        }
    }

    fn mock_user_model(id: i64, email: &str, hashed_pass: &str, enabled: bool) -> user_entity::Model {
        user_entity::Model {
            id,
            uuid: Uuid::new_v4(),
            name: Some("Test User".to_string()),
            email: email.to_string(),
            password: hashed_pass.to_string(),
            first_login: false,
            enabled,
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
            name: "Test Customer".to_string(),
            email: "cust@example.com".to_string(),
            cpf: cpf.map(|s| s.to_string()),
            phone: Some("11999999999".to_string()),
            marketing_consent: true,
            consent_at: Some(Utc::now().naive_utc()),
            active: true,
            created_at: Utc::now().naive_utc(),
            created_by: None,
            updated_at: Utc::now().naive_utc(),
            updated_by: None,
        }
    }

    #[tokio::test]
    async fn test_login_empty_email_or_password() {
        setup_env();
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

        let err1 = AuthenticationUseCase::execute(&db, "".to_string(), "pass123".to_string())
            .await
            .unwrap_err();
        assert_eq!(err1.message, "Email and password are required");

        let err2 = AuthenticationUseCase::execute(&db, "user@test.com".to_string(), "".to_string())
            .await
            .unwrap_err();
        assert_eq!(err2.message, "Email and password are required");
    }

    #[tokio::test]
    async fn test_login_invalid_identifier_without_at_or_digits() {
        setup_env();
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

        let err = AuthenticationUseCase::execute(&db, "plaininvalididentifier".to_string(), "pass".to_string())
            .await
            .unwrap_err();
        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_email_success() {
        setup_env();
        let raw_password = "SecretPassword123!";
        let hashed = bcrypt::hash(raw_password, bcrypt::DEFAULT_COST).unwrap();
        let user = mock_user_model(10, "alice@example.com", &hashed, true);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user]])
            .into_connection();

        let token = AuthenticationUseCase::execute(
            &db,
            "alice@example.com".to_string(),
            raw_password.to_string(),
        )
        .await
        .expect("login should succeed");

        assert_eq!(token.email, "alice@example.com");
        assert_eq!(token.user_id, 10);
        assert_eq!(token.token_type, "Bearer");
        assert!(token.refresh_token.is_some());
    }

    #[tokio::test]
    async fn test_login_email_wrong_password() {
        setup_env();
        let hashed = bcrypt::hash("CorrectPass", bcrypt::DEFAULT_COST).unwrap();
        let user = mock_user_model(10, "alice@example.com", &hashed, true);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user]])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "alice@example.com".to_string(),
            "WrongPass".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_email_disabled_account() {
        setup_env();
        let hashed = bcrypt::hash("Pass123", bcrypt::DEFAULT_COST).unwrap();
        let user = mock_user_model(10, "disabled@example.com", &hashed, false);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user]])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "disabled@example.com".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_email_not_found() {
        setup_env();
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "notfound@example.com".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_email_database_error() {
        setup_env();
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("Connection reset".to_string())])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "error@example.com".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_cpf_success() {
        setup_env();
        let raw_password = "CpfPassword456!";
        let hashed = bcrypt::hash(raw_password, bcrypt::DEFAULT_COST).unwrap();
        let customer = mock_customer_model(5, Some(20), Some("12345678909"));
        let user = mock_user_model(20, "customer@example.com", &hashed, true);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![customer]])
            .append_query_results([vec![user]])
            .into_connection();

        let token = AuthenticationUseCase::execute(
            &db,
            "123.456.789-09".to_string(),
            raw_password.to_string(),
        )
        .await
        .expect("login with formatted CPF should succeed");

        assert_eq!(token.email, "customer@example.com");
        assert_eq!(token.user_id, 20);
    }

    #[tokio::test]
    async fn test_login_cpf_customer_not_found() {
        setup_env();
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<customer_entity::Model>::new()])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "12345678909".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_cpf_customer_without_user_id() {
        setup_env();
        let customer = mock_customer_model(5, None, Some("12345678909"));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![customer]])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "12345678909".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_cpf_user_not_found() {
        setup_env();
        let customer = mock_customer_model(5, Some(20), Some("12345678909"));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![customer]])
            .append_query_results([Vec::<user_entity::Model>::new()])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "12345678909".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_cpf_user_disabled() {
        setup_env();
        let hashed = bcrypt::hash("Pass123", bcrypt::DEFAULT_COST).unwrap();
        let customer = mock_customer_model(5, Some(20), Some("12345678909"));
        let user = mock_user_model(20, "disabled@example.com", &hashed, false);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![customer]])
            .append_query_results([vec![user]])
            .into_connection();

        let err = AuthenticationUseCase::execute(
            &db,
            "12345678909".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();

        assert_eq!(err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_login_cpf_database_errors() {
        setup_env();
        // Error on customer lookup
        let db_err1 = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_errors([DbErr::Custom("db dead".to_string())])
            .into_connection();

        let err1 = AuthenticationUseCase::execute(
            &db_err1,
            "12345678909".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();
        assert_eq!(err1.message, "Invalid credentials");

        // Error on user lookup
        let customer = mock_customer_model(5, Some(20), Some("12345678909"));
        let db_err2 = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![customer]])
            .append_query_errors([DbErr::Custom("user table error".to_string())])
            .into_connection();

        let err2 = AuthenticationUseCase::execute(
            &db_err2,
            "12345678909".to_string(),
            "Pass123".to_string(),
        )
        .await
        .unwrap_err();
        assert_eq!(err2.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_validate_token_success_and_failures() {
        setup_env();
        let user = mock_user_model(10, "carol@example.com", "pass", true);
        let domain_user = UserEntityMapper::from_model(user.clone());
        let token_obj = AuthenticationUseCase::generate_access_token(domain_user);

        // Success: enabled user found
        let db_success = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user.clone()]])
            .into_connection();

        let validated_user = AuthenticationUseCase::validate(&db_success, token_obj.access_token.clone())
            .await
            .expect("token should be valid");
        assert_eq!(validated_user.email, "carol@example.com");

        // Failure: invalid token string
        let db_dummy = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let invalid_err = AuthenticationUseCase::validate(&db_dummy, "invalid.jwt.token".to_string())
            .await
            .unwrap_err();
        assert_eq!(invalid_err.message, "Token is invalid");

        // Failure: user disabled in DB
        let mut disabled_user = user.clone();
        disabled_user.enabled = false;
        let db_disabled = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![disabled_user]])
            .into_connection();
        let dis_err = AuthenticationUseCase::validate(&db_disabled, token_obj.access_token.clone())
            .await
            .unwrap_err();
        assert_eq!(dis_err.message, "Invalid credentials");

        // Failure: user not found
        let db_not_found = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([Vec::<user_entity::Model>::new()])
            .into_connection();
        let not_found_err = AuthenticationUseCase::validate(&db_not_found, token_obj.access_token)
            .await
            .unwrap_err();
        assert_eq!(not_found_err.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_refresh_token_flow() {
        setup_env();
        let user = mock_user_model(10, "dan@example.com", "pass", true);
        let domain_user = UserEntityMapper::from_model(user.clone());
        let access_token_obj = AuthenticationUseCase::generate_access_token(domain_user);
        let refresh_token = access_token_obj.refresh_token.unwrap();

        // Valid refresh token
        let db_ok = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![user]])
            .into_connection();

        let new_token = AuthenticationUseCase::refresh_token(&db_ok, refresh_token)
            .await
            .expect("refresh should succeed");
        assert_eq!(new_token.email, "dan@example.com");

        // Invalid refresh token
        let db_dummy = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let err = AuthenticationUseCase::refresh_token(&db_dummy, "gibberish".to_string())
            .await
            .unwrap_err();
        assert_eq!(err.message, "Token is invalid");
    }
}
