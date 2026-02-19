use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, delete, patch},
    Router,
};
use uuid::Uuid;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;
use tokio::sync::Mutex;
use validator::Validate;

use crate::{
    auth::AuthState,
    db::Database,
    task_queue::TaskQueue,
    webhooks::WebhookService,
    storage::S3Storage,
    rate_limit::RateLimitState,
    models::{
        ConvertResponse,
        StatusResponse,
        DocumentResponse,
        HealthResponse,
        ConversionOptions,
        ConversionProfile,
        CreateProfileRequest,
        UpdateProfileRequest,
        JobStatus,
        ApiError,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub task_queue: Arc<Mutex<TaskQueue>>,
    pub webhook_service: WebhookService,
    pub storage: Arc<S3Storage>,
    pub auth_state: AuthState,
    pub rate_limit_state: RateLimitState,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        submit_conversion,
        get_status,
        get_document,
        delete_job,
        health_check,
        create_profile,
        get_profile,
        list_profiles,
        update_profile,
        delete_profile,
    ),
    components(
        schemas(
            ConvertResponse,
            StatusResponse,
            DocumentResponse,
            HealthResponse,
            ConversionOptions,
            ConversionProfile,
            CreateProfileRequest,
            UpdateProfileRequest,
            JobStatus,
            ApiError,
        )
    ),
    tags(
        (name = "ode", description = "ODE PDF Conversion API")
    )
)]
pub struct ApiDoc;

pub fn create_swagger_router() -> Router<AppState> {
    let api_docs = ApiDoc::openapi();
    SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", api_docs)
        .into()
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/v1/convert", post(submit_conversion))
        .route("/v1/status/:id", get(get_status))
        .route("/v1/documents/:id", get(get_document))
        .route("/v1/jobs/:id", delete(delete_job))
        .route("/v1/profiles", post(create_profile))
        .route("/v1/profiles", get(list_profiles))
        .route("/v1/profiles/:id", get(get_profile))
        .route("/v1/profiles/:id", patch(update_profile))
        .route("/v1/profiles/:id", delete(delete_profile))
}

#[utoipa::path(
    post,
    path = "/v1/convert",
    request_body(content = Option<String>, description = "Multipart form data with 'file' field", content_type = "multipart/form-data"),
    responses(
        (status = 202, description = "Job accepted for processing", body = ConvertResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 415, description = "Unsupported media type", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn submit_conversion(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let mut file_name = None;
    let mut file_data = Vec::new();
    let mut config: Option<ConversionOptions> = None;
    let mut profile_id: Option<Uuid> = None;
    let mut webhook_url: Option<String> = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("invalid_request", format!("Failed to parse multipart: {}", e)))
            )
        })?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                
                file_data = field.bytes().await
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ApiError::new("file_read_error", format!("Failed to read file: {}", e)))
                        )
                    })?
                    .to_vec();
            },
            "config" => {
                let config_str = field.text().await
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ApiError::new("config_parse_error", format!("Failed to read config: {}", e)))
                        )
                    })?;
                
                config = serde_json::from_str(&config_str).ok();
            },
            "profile_id" => {
                let profile_str = field.text().await
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ApiError::new("profile_parse_error", format!("Failed to read profile ID: {}", e)))
                        )
                    })?;
                
                profile_id = serde_json::from_str(&profile_str).ok();
            },
            "webhook_url" => {
                webhook_url = Some(field.text().await
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ApiError::new("webhook_parse_error", format!("Failed to read webhook URL: {}", e)))
                        )
                    })?);
            },
            _ => {}
        }
    }

    let file_name = file_name.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("missing_field", "No file provided"))
        )
    })?;

    if file_data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("empty_file", "Uploaded file is empty"))
        ));
    }

    if !is_valid_pdf(&file_data) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ApiError::new("invalid_file", "Uploaded file is not a valid PDF"))
        ));
    }

    let (final_config, final_profile_id) = if let Some(pid) = profile_id {
        let profile = state.db.get_profile(pid).await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::new("database_error", format!("Failed to fetch profile: {}", e)))
                )
            })?;

        match profile {
            Some(p) => (p.config, Some(pid)),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiError::new("profile_not_found", format!("Profile {} not found", pid)))
                ));
            }
        }
    } else {
        (config.unwrap_or_default(), None)
    };

    if let Err(errors) = final_config.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::with_details(
                "validation_error",
                "Invalid conversion options",
                errors.to_string()
            ))
        ));
    }

    let job_id = Uuid::new_v4();
    let config_json = serde_json::to_value(&final_config)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("serialization_error", format!("Failed to serialize config: {}", e)))
            )
        })?;

    state.db.create_job(
        job_id,
        file_name,
        file_data.len() as u64,
        &file_data,
        config_json,
        webhook_url.clone(),
        final_profile_id,
    ).await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::new("database_error", format!("Failed to create job: {}", e)))
        )
    })?;

    {
        let mut queue = state.task_queue.lock().await;
        queue.enqueue_job(job_id).await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::new("queue_error", format!("Failed to enqueue job: {}", e)))
                )
            })?;
    }

    let response = ConvertResponse {
        job_id,
        status: JobStatus::Pending,
        result_url: None,
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/v1/status/{id}",
    params(
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job status retrieved", body = StatusResponse),
        (status = 404, description = "Job not found", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn get_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let job_metadata = state.db.get_job(id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to fetch job: {}", e)))
            )
        })?;

    match job_metadata {
        Some(metadata) => {
            let response = StatusResponse {
                job_id: metadata.id,
                status: metadata.status,
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                file_name: metadata.file_name,
                progress: None,
                error_message: metadata.error_message,
            };
            Ok((StatusCode::OK, Json(response)))
        },
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("job_not_found", format!("Job {} not found", id)))
        )),
    }
}

#[utoipa::path(
    get,
    path = "/v1/documents/{id}",
    params(
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Document retrieved", body = DocumentResponse),
        (status = 404, description = "Document not found", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let job_metadata = state.db.get_job(id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to fetch job: {}", e)))
            )
        })?;

    match job_metadata {
        Some(metadata) => {
            if metadata.status != JobStatus::Completed {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiError::new("job_not_ready", format!("Job {} is not completed", id)))
                ));
            }

            let result_url = metadata.result_url.ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiError::new("result_not_found", "Conversion result not available"))
                )
            })?;

            let html_content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <meta charset="UTF-8">
</head>
<body>
    <iframe src="{}" style="width:100%; height:100vh; border:none;"></iframe>
</body>
</html>"#, metadata.file_name, result_url);

            let response = DocumentResponse {
                job_id: metadata.id,
                html_content,
                css_content: None,
                page_count: 1,
                created_at: metadata.created_at,
            };

            Ok((StatusCode::OK, Json(response)))
        },
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("job_not_found", format!("Job {} not found", id)))
        )),
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service healthy", body = HealthResponse)
    ),
    tag = "ode"
)]
pub async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    };
    (StatusCode::OK, Json(response))
}

pub async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "ODE - Oxidized Document Engine",
        "version": env!("CARGO_PKG_VERSION"),
        "docs": "/docs",
        "health": "/health",
        "endpoints": {
            "convert": "POST /v1/convert",
            "status": "GET /v1/status/:id",
            "document": "GET /v1/documents/:id",
            "register": "POST /auth/register",
            "login": "POST /auth/login"
        }
    }))
}

pub async fn convert_sync(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let mut file_data = Vec::new();
    let mut _file_name = String::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| {
            (StatusCode::BAD_REQUEST, Json(ApiError::new("invalid_request", format!("Failed to parse multipart: {}", e))))
        })?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            _file_name = field.file_name().unwrap_or("upload.pdf").to_string();
            file_data = field.bytes().await
                .map_err(|e| {
                    (StatusCode::BAD_REQUEST, Json(ApiError::new("file_read_error", format!("Failed to read file: {}", e))))
                })?
                .to_vec();
        }
    }

    if file_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ApiError::new("empty_file", "No file provided"))));
    }

    if !is_valid_pdf(&file_data) {
        return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, Json(ApiError::new("invalid_file", "Not a valid PDF file"))));
    }

    let config = ode_core::ConversionConfig::default();
    let result = ode_core::convert_pdf(&file_data, &config)
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new("conversion_error", format!("PDF conversion failed: {}", e))))
        })?;

    let mut combined_html = String::new();
    let mut combined_css = String::new();

    for page in &result.pages {
        if !page.css.is_empty() {
            combined_css.push_str(&page.css);
            combined_css.push('\n');
        }
    }

    combined_html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<style>\n");
    combined_html.push_str("* { margin:0; padding:0; box-sizing:border-box; }\n");
    combined_html.push_str("body { background:#f0f0f0; padding:20px 0; }\n");
    combined_html.push_str(".page-wrapper { width:100%; max-width:1480px; margin:20px auto; }\n");
    combined_html.push_str(".page { transform-origin:top left; box-shadow:0 2px 8px rgba(0,0,0,0.15); }\n");
    combined_html.push_str(&combined_css);
    if !result.css.is_empty() {
        combined_html.push_str(&result.css);
    }
    combined_html.push_str("\n</style>\n</head>\n<body>\n");

    for page in &result.pages {
        let bg = page.background_color.as_deref().unwrap_or("white");
        // Wrap each page in a container that scales it to fit the viewport
        combined_html.push_str(&format!(
            "<div class=\"page-wrapper\" style=\"aspect-ratio:{}/{};\"><div class=\"page\" id=\"page-{}\" style=\"width:{}px;height:{}px;position:relative;background:{};overflow:hidden;transform:scale(var(--s));\" data-w=\"{}\">\n",
            page.width, page.height, page.page_number, page.width, page.height, bg, page.width
        ));
        combined_html.push_str(&page.html);
        combined_html.push_str("\n</div></div>\n");
    }

    combined_html.push_str("<script>\n");
    combined_html.push_str("function resize(){document.querySelectorAll('.page-wrapper').forEach(w=>{const p=w.querySelector('.page');const s=Math.min(1,w.clientWidth/parseFloat(p.dataset.w));p.style.setProperty('--s',s);w.style.height=(parseFloat(p.style.height)*s)+'px';});}\n");
    combined_html.push_str("window.addEventListener('resize',resize);resize();\n");
    combined_html.push_str("</script>\n");
    combined_html.push_str("</body>\n</html>");

    Ok(axum::response::Html(combined_html))
}

pub async fn web_ui() -> impl IntoResponse {
    axum::response::Html(include_str!("ui.html"))
}

pub fn is_valid_pdf(data: &[u8]) -> bool {
    data.len() >= 5 && &data[0..5] == b"%PDF-"
}

#[utoipa::path(
    delete,
    path = "/v1/jobs/{id}",
    params(
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 204, description = "Job deleted successfully"),
        (status = 404, description = "Job not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let job_metadata = state.db.get_job(id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to fetch job: {}", e)))
            )
        })?;

    match job_metadata {
        Some(_) => {
            state.db.delete_job(id).await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiError::new("database_error", format!("Failed to delete job: {}", e)))
                    )
                })?;

            state.storage.delete_job_assets(id).await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiError::new("storage_error", format!("Failed to delete assets: {}", e)))
                    )
                })?;

            Ok(StatusCode::NO_CONTENT)
        },
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("job_not_found", format!("Job {} not found", id)))
        )),
    }
}

#[utoipa::path(
    post,
    path = "/v1/profiles",
    request_body = CreateProfileRequest,
    responses(
        (status = 201, description = "Profile created", body = ConversionProfile),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn create_profile(
    State(state): State<AppState>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    if let Err(errors) = request.config.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::with_details(
                "validation_error",
                "Invalid conversion options",
                errors.to_string()
            ))
        ));
    }

    let profile = state.db.create_profile(request).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to create profile: {}", e)))
            )
        })?;

    Ok((StatusCode::CREATED, Json(profile)))
}

#[utoipa::path(
    get,
    path = "/v1/profiles/{id}",
    params(
        ("id" = Uuid, Path, description = "Profile ID")
    ),
    responses(
        (status = 200, description = "Profile retrieved", body = ConversionProfile),
        (status = 404, description = "Profile not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let profile = state.db.get_profile(id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to fetch profile: {}", e)))
            )
        })?;

    match profile {
        Some(profile) => Ok((StatusCode::OK, Json(profile))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("profile_not_found", format!("Profile {} not found", id)))
        )),
    }
}

#[utoipa::path(
    get,
    path = "/v1/profiles",
    responses(
        (status = 200, description = "Profiles listed", body = Vec<ConversionProfile>),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn list_profiles(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let profiles = state.db.list_profiles().await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to list profiles: {}", e)))
            )
        })?;

    Ok((StatusCode::OK, Json(profiles)))
}

#[utoipa::path(
    patch,
    path = "/v1/profiles/{id}",
    params(
        ("id" = Uuid, Path, description = "Profile ID")
    ),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = ConversionProfile),
        (status = 404, description = "Profile not found", body = ApiError),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    if let Some(ref config) = request.config {
        if let Err(errors) = config.validate() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError::with_details(
                    "validation_error",
                    "Invalid conversion options",
                    errors.to_string()
                ))
            ));
        }
    }

    let profile = state.db.update_profile(id, request).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to update profile: {}", e)))
            )
        })?;

    match profile {
        Some(profile) => Ok((StatusCode::OK, Json(profile))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("profile_not_found", format!("Profile {} not found", id)))
        )),
    }
}

#[utoipa::path(
    delete,
    path = "/v1/profiles/{id}",
    params(
        ("id" = Uuid, Path, description = "Profile ID")
    ),
    responses(
        (status = 204, description = "Profile deleted successfully"),
        (status = 404, description = "Profile not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "ode"
)]
pub async fn delete_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let deleted = state.db.delete_profile(id).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", format!("Failed to delete profile: {}", e)))
            )
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("profile_not_found", format!("Profile {} not found", id)))
        ))
    }
}