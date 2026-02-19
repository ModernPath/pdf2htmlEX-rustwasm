use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use std::env;
use tracing::{warn, trace};
use uuid::Uuid;

use crate::models::{User, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new() -> Result<Self, String> {
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }

    pub fn hash_password(&self, password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password: {}", e))
            .map(|hash| hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, String> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| format!("Invalid password hash format: {}", e))?;
        let argon2 = Argon2::default();

        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map(|_| true)
            .map_err(|e| format!("Password verification failed: {}", e))
    }

    pub fn generate_token(&self, user: &User) -> Result<String, String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);

        let claims = Claims {
            sub: user.id.to_string(),
            user_id: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| format!("Failed to generate token: {}", e))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| format!("Token validation failed: {}", e))
    }

    pub fn hash_api_key(&self, api_key: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(api_key.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash API key: {}", e))
            .map(|hash| hash.to_string())
    }

    pub fn verify_api_key(&self, api_key: &str, hash: &str) -> Result<bool, String> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| format!("Invalid API key hash format: {}", e))?;
        let argon2 = Argon2::default();

        argon2
            .verify_password(api_key.as_bytes(), &parsed_hash)
            .map(|_| true)
            .map_err(|e| format!("API key verification failed: {}", e))
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new().expect("Failed to initialize AuthService")
    }
}

pub struct AuthContext {
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub role: Option<Role>,
    pub is_api_key: bool,
}

#[derive(Clone)]
pub struct AuthState {
    pub auth_service: AuthService,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            auth_service: AuthService::new().expect("Failed to initialize AuthState"),
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn jwt_middleware(
    State(auth_state): State<AuthState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if auth_header.is_none() {
        warn!("No Authorization header provided");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth_str = auth_header.unwrap();
    let token = match auth_str.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            warn!("Invalid Authorization header format");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let claims = auth_state.auth_service.validate_token(token)
        .map_err(|e| {
            warn!("JWT validation failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let role = match claims.role.as_str() {
        "Admin" => Role::Admin,
        "Developer" => Role::Developer,
        "Viewer" => Role::Viewer,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let email = claims.email.clone();
    req.extensions_mut().insert(claims);
    req.extensions_mut().insert(role);

    trace!("JWT validated successfully for user: {}", email);
    Ok(next.run(req).await)
}

pub fn rbac_middleware(required_role: Role) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |req: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let user_role = req.extensions().get::<Role>();

            match user_role {
                Some(role) if role.has_permission(&required_role) => {
                    trace!("RBAC check passed: {:?} has permission for {:?}", role, required_role);
                    Ok(next.run(req).await)
                }
                Some(_role) => {
                    warn!("RBAC check failed: does not have permission for {:?}", required_role);
                    Err(StatusCode::FORBIDDEN)
                }
                None => {
                    warn!("RBAC check failed: No role found in request extensions");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        })
    }
}

pub async fn api_key_middleware(
    State(app_state): State<crate::routes::AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok());

    let provided_key = auth_header.ok_or_else(|| {
        warn!("No API key provided");
        StatusCode::UNAUTHORIZED
    })?;

    let db_keys = app_state.db.list_all_active_api_keys().await
        .map_err(|e| {
            warn!("Failed to fetch API keys from database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    for key_info in db_keys {
        let key_id = key_info.0;
        let key_hash = &key_info.1;

        if app_state.auth_state.auth_service.verify_api_key(provided_key, key_hash).unwrap_or(false) {
            let user_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT user_id FROM api_keys WHERE id = $1 AND is_active = true"
            )
            .bind(key_id)
            .fetch_optional(app_state.db.pool())
            .await
            .map_err(|e| {
                warn!("Failed to query API key user_id: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            if let Some(uid) = user_id {
                if let Ok(Some(user)) = app_state.db.get_user_by_id(uid).await {
                    req.extensions_mut().insert(user.role.clone());
                    req.extensions_mut().insert(user.id.to_string());
                    req.extensions_mut().insert(user.email.clone());

                    trace!("API key validated successfully for user: {}", user.email);
                    return Ok(next.run(req).await);
                } else {
                    warn!("API key with id {} exists but associated user {} not found", key_id, uid);
                }
            }
        }
    }

    warn!("Invalid API key provided");
    Err(StatusCode::UNAUTHORIZED)
}

pub fn get_valid_api_key() -> String {
    env::var("ODE_API_KEY").unwrap_or_else(|_| "default-api-key".to_string())
}