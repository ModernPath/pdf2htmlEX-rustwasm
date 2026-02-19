use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

use ode_api::{
    db::Database,
    task_queue::TaskQueue,
    webhooks::WebhookService,
    routes::AppState,
    models::{ConversionOptions, JobStatus},
};

#[tokio::test]
async fn test_conversion_options_defaults() {
    let options = ConversionOptions::default();
    
    assert_eq!(options.embed_css, true);
    assert_eq!(options.embed_font, true);
    assert_eq!(options.embed_image, true);
    assert_eq!(options.embed_javascript, false);
    assert_eq!(options.correct_text_visibility, true);
    assert_eq!(options.page_range, None);
    assert_eq!(options.dpi, None);
    assert_eq!(options.zoom, None);
}

#[tokio::test]
async fn test_job_status_display() {
    assert_eq!(JobStatus::Pending.to_string(), "pending");
    assert_eq!(JobStatus::Processing.to_string(), "processing");
    assert_eq!(JobStatus::Completed.to_string(), "completed");
    assert_eq!(JobStatus::Failed.to_string(), "failed");
}

#[tokio::test]
async fn test_api_error_creation() {
    let error = ode_api::models::ApiError::new("test_error", "Test error message");
    
    assert_eq!(error.error, "test_error");
    assert_eq!(error.message, "Test error message");
    assert_eq!(error.details, None);
}

#[tokio::test]
async fn test_api_error_with_details() {
    let error = ode_api::models::ApiError::with_details(
        "test_error",
        "Test error message",
        "Additional details"
    );
    
    assert_eq!(error.error, "test_error");
    assert_eq!(error.message, "Test error message");
    assert_eq!(error.details, Some("Additional details".to_string()));
}

#[tokio::test]
async fn test_webhook_service_creation() {
    let service = WebhookService::new();
    
    // Just verify it creates without error
    // Actual webhook testing requires an HTTP server
    assert!(true);
}

#[tokio::test]
async fn test_conversion_options_serialization() {
    let options = ConversionOptions::default();
    let json = serde_json::to_value(&options).unwrap();
    
    assert!(json.is_object());
    assert_eq!(json["embed_css"], true);
    assert_eq!(json["embed_font"], true);
}

#[tokio::test]
async fn test_pdf_validation() {
    // Valid PDF header
    let valid_pdf = b"%PDF-1.4\n%...";
    assert!(ode_api::routes::is_valid_pdf(valid_pdf));

    // Invalid PDF header
    let invalid_pdf = b"Not a PDF";
    assert!(!ode_api::routes::is_valid_pdf(invalid_pdf));

    // Too short
    let short_pdf = b"%PDF";
    assert!(!ode_api::routes::is_valid_pdf(short_pdf));
}