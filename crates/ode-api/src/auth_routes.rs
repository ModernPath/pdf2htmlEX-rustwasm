use axum::{
    extract::{Path, State},
    http::{
        header::{SET_COOKIE, CONTENT_TYPE},
        StatusCode,
    },
    middleware,
    response::{IntoResponse, Json, Response},
    routing::{get, post, delete},
    Router,
};
use sqlx;
use getrandom::getrandom;
use hex;
use uuid::Uuid;
use validator::Validate;

static SERVER_START_TIME: std::sync::LazyLock<std::time::Instant> = std::sync::LazyLock::new(std::time::Instant::now);

use crate::{
    auth::{jwt_middleware, rbac_middleware, AuthState},
    routes::{create_router, AppState},
    rate_limit::{rate_limit_middleware, RateLimitState},
    models::{
        CreateUserRequest,
        LoginRequest,
        LoginResponse,
        CreateApiKeyRequest,
        CreateApiKeyResponse,
        User,
        ApiKey,
        Role,
        SystemStats,
        ApiError,
    },
};

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User registered successfully", body = User),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 409, description = "User already exists", body = ApiError)
    ),
    tag = "authentication"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    if let Err(errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::with_details(
                "validation_error",
                "Invalid registration request",
                errors.to_string()
            ))
        ));
    }

    if let Err(email_error) = request.validate_email() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("invalid_email", email_error))
        ));
    }

    let role = request.role.unwrap_or(Role::Developer);

    let existing_user = state.db.get_user_by_email(&request.email).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to check existing user: {}", e)))
            )
        })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError::new("user_exists", "User already exists"))
        ));
    }

    let password_hash = state.auth_state.auth_service.hash_password(&request.password)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("hash_error", format!("Failed to hash password: {}", e)))
            )
        })?;

    let user = state.db.create_user(request.email.clone(), password_hash, role).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to create user: {}", e)))
            )
        })?;

    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ApiError)
    ),
    tag = "authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    if let Err(errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::with_details(
                "validation_error",
                "Invalid login request",
                errors.to_string()
            ))
        ));
    }

    let user = state.db.get_user_by_email(&request.email).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to fetch user: {}", e)))
            )
        })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError::new("invalid_credentials", "Invalid email or password"))
        )
    })?;

    let is_valid = state.auth_state.auth_service.verify_password(&request.password, &user.password_hash)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("verification_error", format!("Password verification failed: {}", e)))
            )
        })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError::new("invalid_credentials", "Invalid email or password"))
        ));
    }

    let token = state.auth_state.auth_service.generate_token(&user)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("token_error", format!("Failed to generate token: {}", e)))
            )
        })?;

    let token_clone = token.clone();
    let response = LoginResponse {
        token,
        user: User {
            id: user.id,
            email: user.email.clone(),
            password_hash: String::new(),
            role: user.role.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        },
    };

    const COOKIE_NAME: &str = "jwt_token";
    const COOKIE_MAX_AGE: u64 = 86400;

    let cookie_value = format!(
        "{}={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/",
        COOKIE_NAME, token_clone, COOKIE_MAX_AGE
    );

    let response_builder = Response::builder()
        .status(StatusCode::OK)
        .header(SET_COOKIE, cookie_value)
        .header(CONTENT_TYPE, "application/json");

    let body = serde_json::to_vec(&response)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::new("serialization_error", format!("Failed to serialize response: {}", e)))
        ))?;

    Ok(response_builder.body(axum::body::Body::from(body)).unwrap())
}

#[utoipa::path(
    post,
    path = "/auth/users/{user_id}/api-keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created successfully", body = CreateApiKeyResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "authentication"
)]
pub async fn create_api_key(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let mut key_bytes = [0u8; 32];
    getrandom(&mut key_bytes).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::new("random_error", format!("Failed to generate random bytes: {}", e)))
        )
    })?;

    let raw_key = hex::encode(&key_bytes);

    let key_hash = state.auth_state.auth_service.hash_api_key(&raw_key)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("hash_error", format!("Failed to hash API key: {}", e)))
            )
        })?;

    let api_key = state.db.create_api_key(user_id, key_hash, request.name).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to create API key: {}", e)))
            )
        })?;

    let response = CreateApiKeyResponse {
        api_key: raw_key,
        key_info: api_key,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/auth/users/{user_id}/api-keys",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "API keys retrieved", body = Vec<ApiKey>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "authentication"
)]
pub async fn list_api_keys(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let api_keys = state.db.list_api_keys(user_id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to list API keys: {}", e)))
            )
        })?;

    Ok((StatusCode::OK, Json(api_keys)))
}

#[utoipa::path(
    delete,
    path = "/auth/api-keys/{key_id}",
    params(
        ("key_id" = Uuid, Path, description = "API Key ID")
    ),
    responses(
        (status = 204, description = "API key revoked successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "authentication"
)]
pub async fn revoke_api_key(
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    state.db.revoke_api_key(key_id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to revoke API key: {}", e)))
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/admin/system-stats",
    responses(
        (status = 200, description = "System statistics retrieved", body = SystemStats),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin role required"),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "admin"
)]
pub async fn get_system_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let (total_users, total_api_keys, total_jobs, processing_jobs) = state.db.get_system_stats().await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to fetch system stats: {}", e)))
            )
        })?;

    let uptime_seconds = SERVER_START_TIME.elapsed().as_secs() as i64;

    let completed_jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status = 'completed'")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let failed_jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status = 'failed'")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let stats = SystemStats {
        total_jobs,
        completed_jobs,
        failed_jobs,
        processing_jobs,
        total_users,
        total_api_keys,
        uptime_seconds,
    };

    Ok((StatusCode::OK, Json(stats)))
}

pub fn create_auth_router() -> Router<AppState> {
    // Routes that require JWT authentication
    let authenticated_routes = Router::new()
        .route("/auth/users/:user_id/api-keys", post(create_api_key))
        .route("/auth/users/:user_id/api-keys", get(list_api_keys))
        .route("/auth/api-keys/:key_id", delete(revoke_api_key));

    // Admin routes require JWT + Admin role
    let admin_routes = Router::new()
        .route("/admin/system-stats", get(get_system_stats))
        .route_layer(middleware::from_fn(rbac_middleware(Role::Admin)));

    // Public routes (no JWT required)
    let public_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login));

    public_routes
        .merge(authenticated_routes)
        .merge(admin_routes)
}

pub fn create_final_router(state: AppState, rate_limit_state: RateLimitState, auth_state: AuthState) -> axum::Router {
    // Public routes that don't need JWT
    let public_routes = Router::new()
        .route("/", get(crate::routes::root))
        .route("/ui", get(crate::routes::web_ui))
        .route("/v1/convert-sync", post(crate::routes::convert_sync))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/health", get(crate::routes::health_check))
        .merge(crate::routes::create_swagger_router());

    // Authenticated routes with JWT middleware
    let authenticated_routes = create_router()
        .merge(
            Router::new()
                .route("/auth/users/:user_id/api-keys", post(create_api_key))
                .route("/auth/users/:user_id/api-keys", get(list_api_keys))
                .route("/auth/api-keys/:key_id", delete(revoke_api_key))
                .merge(
                    Router::new()
                        .route("/admin/system-stats", get(get_system_stats))
                        .route_layer(middleware::from_fn(rbac_middleware(Role::Admin)))
                )
        )
        .route_layer(middleware::from_fn_with_state(
            auth_state.clone(),
            jwt_middleware
        ));

    public_routes
        .merge(authenticated_routes)
        .route_layer(middleware::from_fn_with_state(
            rate_limit_state.clone(),
            rate_limit_middleware
        ))
        .with_state(state)
}