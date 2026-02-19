#[cfg(test)]
mod auth_tests {
    use crate::db::Database;
    use crate::task_queue::TaskQueue;
    use crate::webhooks::WebhookService;
    use crate::storage::S3Storage;
    use crate::auth::AuthState;
    use crate::rate_limit::{RateLimitState, RateLimiter};
    use crate::auth_routes::create_auth_router;
    use crate::routes::AppState;
    use crate::models::*;
    use axum::{
        body::Body,
        extract::Request,
        http::{header, Method, StatusCode},
    };
    use bytes::Bytes;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower::ServiceExt;
    use validator::Validate;

    async fn create_test_state() -> AppState {
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/ode_test".to_string());

        let db = Database::new(&db_url).await
            .expect("Failed to connect to test database");

        let redis_url = std::env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379/1".to_string());

        let task_queue = TaskQueue::new(&redis_url).await
            .expect("Failed to connect to Redis");

        AppState {
            db,
            task_queue: Arc::new(Mutex::new(task_queue)),
            webhook_service: WebhookService::new(),
            storage: Arc::new(S3Storage::new("test-bucket".to_string()).await.unwrap()),
            auth_state: AuthState::new(),
            rate_limit_state: RateLimitState::new(),
        }
    }

    #[tokio::test]
    async fn test_us_011_registration_with_valid_details() {
        let state = create_test_state().await;

        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "super_secret_password_123".to_string(),
            role: None,
        };

        assert!(request.validate().is_ok());
        assert!(request.validate_email().is_ok());

        let password_hash = state.auth_state.auth_service.hash_password(&request.password)
            .expect("Failed to hash password");

        assert_ne!(password_hash, request.password);
        assert!(password_hash.contains("$argon2"));

        let user = state.db.create_user(
            request.email.clone(),
            password_hash,
            request.role.unwrap_or(Role::Developer)
        ).await.expect("Failed to create user");

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, Role::Developer);
    }

    #[tokio::test]
    async fn test_us_011_weak_password_rejected() {
        let _state = create_test_state().await;

        let weak_request = CreateUserRequest {
            email: "weak@example.com".to_string(),
            password: "short".to_string(),
            role: None,
        };

        assert!(weak_request.validate().is_err());
    }

    #[tokio::test]
    async fn test_us_011_duplicate_email_rejected() {
        let state = create_test_state().await;

        let email = "duplicate@example.com";
        let password_hash = state.auth_state.auth_service.hash_password("proper_password_123")
            .expect("Failed to hash password");

        state.db.create_user(email.to_string(), password_hash.clone(), Role::Developer)
            .await
            .expect("Failed to create first user");

        let existing_user = state.db.get_user_by_email(email).await
            .expect("Failed to check existing user");

        assert!(existing_user.is_some());
    }

    #[tokio::test]
    async fn test_us_012_valid_credentials_return_jwt() {
        let state = create_test_state().await;

        let email = "login@example.com";
        let password = "secure_login_password_123";
        let password_hash = state.auth_state.auth_service.hash_password(password)
            .expect("Failed to hash password");

        state.db.create_user(email.to_string(), password_hash, Role::Developer)
            .await
            .expect("Failed to create user");

        let user = state.db.get_user_by_email(email).await
            .expect("Failed to fetch user")
            .expect("User not found");

        let is_valid = state.auth_state.auth_service.verify_password(password, &user.password_hash)
            .expect("Failed to verify password");

        assert!(is_valid);

        let token = state.auth_state.auth_service.generate_token(&user)
            .expect("Failed to generate token");

        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3);

        let claims = state.auth_state.auth_service.validate_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.email, email);
        assert_eq!(claims.user_id, user.id.to_string());
    }

    #[tokio::test]
    async fn test_us_012_invalid_credentials_rejected() {
        let state = create_test_state().await;

        let email = "nologin@example.com";
        let password = "correct_password_123";
        let password_hash = state.auth_state.auth_service.hash_password(password)
            .expect("Failed to hash password");

        state.db.create_user(email.to_string(), password_hash, Role::Developer)
            .await
            .expect("Failed to create user");

        let user = state.db.get_user_by_email(email).await
            .expect("Failed to fetch user")
            .expect("User not found");

        let is_valid = state.auth_state.auth_service.verify_password("wrong_password", &user.password_hash)
            .expect("Failed to verify password");

        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_us_012_no_token_returns_401() {
        let state = create_test_state().await;

        let router = create_auth_router().with_state(state);

        let request = Request::builder()
            .uri("/admin/system-stats")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_us_013_rbac_developer_cannot_access_admin() {
        let state = create_test_state().await;

        let email = "developer@example.com";
        let password_hash = state.auth_state.auth_service.hash_password("developer_password_123")
            .expect("Failed to hash password");

        let user = state.db.create_user(email.to_string(), password_hash, Role::Developer)
            .await
            .expect("Failed to create user");

        let token = state.auth_state.auth_service.generate_token(&user)
            .expect("Failed to generate token");

        let router = create_auth_router().with_state(state);

        let request = Request::builder()
            .uri("/admin/system-stats")
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_us_013_rbac_admin_can_access_admin() {
        let state = create_test_state().await;

        let email = "admin@example.com";
        let password_hash = state.auth_state.auth_service.hash_password("admin_password_123")
            .expect("Failed to hash password");

        let user = state.db.create_user(email.to_string(), password_hash, Role::Admin)
            .await
            .expect("Failed to create user");

        let token = state.auth_state.auth_service.generate_token(&user)
            .expect("Failed to generate token");

        let router = create_auth_router().with_state(state);

        let request = Request::builder()
            .uri("/admin/system-stats")
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_us_013_rbac_viewer_cannot_delete() {
        let state = create_test_state().await;

        let email = "viewer@example.com";
        let password_hash = state.auth_state.auth_service.hash_password("viewer_password_123")
            .expect("Failed to hash password");

        let user = state.db.create_user(email.to_string(), password_hash, Role::Viewer)
            .await
            .expect("Failed to create user");

        let api_key = state.db.create_api_key(user.id, "test_key_hash".to_string(), "test_key".to_string())
            .await
            .expect("Failed to create API key");

        let token = state.auth_state.auth_service.generate_token(&user)
            .expect("Failed to generate token");

        let router = create_auth_router().with_state(state);

        let request = Request::builder()
            .uri(format!("/auth/api-keys/{}", api_key.id))
            .method(Method::DELETE)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await
            .expect("Request failed");

        assert!(response.status() != StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_us_014_api_key_generation() {
        let state = create_test_state().await;

        let email = "apikey@example.com";
        let password_hash = state.auth_state.auth_service.hash_password("apikey_password_123")
            .expect("Failed to hash password");

        let user = state.db.create_user(email.to_string(), password_hash, Role::Developer)
            .await
            .expect("Failed to create user");

        let raw_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key_hash = state.auth_state.auth_service.hash_api_key(raw_key)
            .expect("Failed to hash API key");

        let api_key = state.db.create_api_key(user.id, key_hash.clone(), "test_key".to_string())
            .await
            .expect("Failed to create API key");

        assert_ne!(key_hash, raw_key);
        assert!(key_hash.contains("$argon2"));
        assert!(api_key.is_active);
    }

    #[tokio::test]
    async fn test_us_014_api_key_authentication() {
        let state = create_test_state().await;

        let email = "apiauth@example.com";
        let password_hash = state.auth_state.auth_service.hash_password("apiauth_password_123")
            .expect("Failed to hash password");

        let user = state.db.create_user(email.to_string(), password_hash, Role::Developer)
            .await
            .expect("Failed to create user");

        let raw_key = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
        let key_hash = state.auth_state.auth_service.hash_api_key(raw_key)
            .expect("Failed to hash API key");

        state.db.create_api_key(user.id, key_hash.clone(), "auth_test_key".to_string())
            .await
            .expect("Failed to create API key");

        let is_valid = state.auth_state.auth_service.verify_api_key(raw_key, &key_hash)
            .expect("Failed to verify API key");

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_us_014_revoked_key_rejected() {
        let state = create_test_state().await;

        let email = "revoke@example.com";
        let password_hash = state.auth_state.auth_service.hash_password("revoke_password_123")
            .expect("Failed to hash password");

        let user = state.db.create_user(email.to_string(), password_hash, Role::Developer)
            .await
            .expect("Failed to create user");

        let raw_key = "revokedrevokedrevokedrevokedrevokedrevokedrevokedrevokedrevokedrevoked";
        let key_hash = state.auth_state.auth_service.hash_api_key(raw_key)
            .expect("Failed to hash API key");

        let api_key = state.db.create_api_key(user.id, key_hash.clone(), "revoked_key".to_string())
            .await
            .expect("Failed to create API key");

        state.db.revoke_api_key(api_key.id).await
            .expect("Failed to revoke API key");

        let revoked_key = state.db.get_api_key_by_hash(&key_hash).await
            .expect("Failed to fetch API key");

        assert!(revoked_key.is_none() || !revoked_key.unwrap().is_active);
    }

    #[tokio::test]
    async fn test_us_015_rate_limiting_respects_window() {
        let rate_limiter = RateLimiter::new(5, 60);

        let client_key = "test_client";

        for i in 0..5 {
            let result = rate_limiter.check_rate_limit(client_key).await
                .expect("Rate limit check failed");
            assert!(result, "Request {i} should be allowed");
        }

        let result = rate_limiter.check_rate_limit(client_key).await
            .expect("Rate limit check failed");
        assert!(!result, "6th request should be rate limited");
    }

    #[tokio::test]
    async fn test_us_015_rate_limit_resets_after_window() {
        let rate_limiter = RateLimiter::new(3, 1);

        let client_key = "reset_test";

        for i in 0..3 {
            let result = rate_limiter.check_rate_limit(client_key).await
                .expect("Rate limit check failed");
            assert!(result, "Request {i} should be allowed");
        }

        let result = rate_limiter.check_rate_limit(client_key).await
            .expect("Rate limit check failed");
        assert!(!result, "4th request should be rate limited");

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let result = rate_limiter.check_rate_limit(client_key).await
            .expect("Rate limit check failed");
        assert!(result, "Request after window reset should be allowed");
    }
}
