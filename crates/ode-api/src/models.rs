use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Processing => write!(f, "processing"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobMetadata {
    pub id: Uuid,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub file_name: String,
    pub file_size: u64,
    pub webhook_url: Option<String>,
    pub error_message: Option<String>,
    pub profile_id: Option<Uuid>,
    pub result_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConvertRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ConversionOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct ConversionOptions {
    #[serde(default)]
    pub page_range: Option<(u32, u32)>,
    #[serde(default)]
    #[validate(range(min = 1.0, message = "DPI must be greater than 0"))]
    pub dpi: Option<f64>,
    #[serde(default)]
    #[validate(range(min = 0.1, max = 10.0, message = "Zoom must be between 0.1 and 10.0"))]
    pub zoom: Option<f64>,
    #[serde(default)]
    pub embed_css: bool,
    #[serde(default)]
    pub embed_font: bool,
    #[serde(default)]
    pub embed_image: bool,
    #[serde(default)]
    pub embed_javascript: bool,
    #[serde(default)]
    pub correct_text_visibility: bool,
    #[serde(default)]
    pub background_format: Option<String>,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            page_range: None,
            dpi: None,
            zoom: None,
            embed_css: true,
            embed_font: true,
            embed_image: true,
            embed_javascript: false,
            correct_text_visibility: true,
            background_format: Some("svg".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConversionProfile {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub config: ConversionOptions,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateProfileRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub config: ConversionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ConversionOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConvertResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatusResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub file_name: String,
    pub progress: Option<f32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentResponse {
    pub job_id: Uuid,
    pub html_content: String,
    pub css_content: Option<String>,
    pub page_count: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookPayload {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        error: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Developer,
    Viewer,
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "developer" => Role::Developer,
            "viewer" => Role::Viewer,
            _ => Role::Viewer,
        }
    }

    pub fn has_permission(&self, required_role: &Role) -> bool {
        match required_role {
            Role::Admin => matches!(self, Role::Admin),
            Role::Developer => matches!(self, Role::Admin | Role::Developer),
            Role::Viewer => matches!(self, Role::Admin | Role::Developer | Role::Viewer),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::Developer => write!(f, "Developer"),
            Role::Viewer => write!(f, "Viewer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateUserRequest {
    pub email: String,
    #[validate(length(min = 12))]
    pub password: String,
    #[serde(default = "default_role")]
    pub role: Option<Role>,
}

fn validate_email(email: &str) -> Result<(), String> {
    if !email.contains('@') || !email.contains('.') || email.len() < 5 {
        return Err("Invalid email format".to_string());
    }
    Ok(())
}

impl CreateUserRequest {
    pub fn validate_email(&self) -> Result<(), String> {
        validate_email(&self.email)
    }
}

fn default_role() -> Option<Role> {
    Some(Role::Developer)
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing)]
    pub key_hash: String,
    pub name: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub api_key: String,
    pub key_info: ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemStats {
    pub total_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
    pub processing_jobs: i64,
    pub total_users: i64,
    pub total_api_keys: i64,
    pub uptime_seconds: i64,
}
