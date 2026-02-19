use reqwest::Client;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{WebhookPayload, JobStatus};
use tracing::{info, warn, error};

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF_MS: u64 = 1000;

#[derive(Clone)]
pub struct WebhookService {
    client: Client,
}

impl WebhookService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn send_webhook(
        &self,
        webhook_url: &str,
        job_id: Uuid,
        status: JobStatus,
        file_name: String,
        result_url: Option<String>,
        error_message: Option<String>,
        created_at: chrono::DateTime<Utc>,
    ) -> Result<(), WebhookError> {
        let payload = WebhookPayload {
            job_id,
            status: status.clone(),
            created_at,
            completed_at: Utc::now(),
            file_name,
            result_url,
            error_message,
            duration_ms: (Utc::now() - created_at).num_milliseconds(),
        };

        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| WebhookError::SerializationFailed(e.to_string()))?;

        let mut attempt = 0;
        let mut backoff = INITIAL_BACKOFF_MS;

        loop {
            attempt += 1;

            match self.attempt_delivery(webhook_url, &payload_json).await {
                Ok(_) => {
                    info!(webhook_url, attempt, "Webhook delivered successfully");
                    return Ok(());
                }
                Err(WebhookError::HttpError(status_code)) if status_code >= 500 => {
                    warn!(
                        webhook_url,
                        attempt, status_code,
                        "Webhook delivery failed with server error, retrying with backoff {}ms",
                        backoff
                    );

                    if attempt >= MAX_RETRIES {
                        error!(
                            webhook_url,
                            attempt,
                            "Webhook delivery failed after {} retries",
                            MAX_RETRIES
                        );
                        return Err(WebhookError::MaxRetriesExceeded);
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                    backoff *= 2;
                }
                Err(e) => {
                    warn!(webhook_url, attempt, error = %e, "Webhook delivery failed permanently");
                    return Err(e);
                }
            }
        }
    }

    async fn attempt_delivery(&self, webhook_url: &str, payload_json: &str) -> Result<(), WebhookError> {
        let response = self
            .client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "ODE-Webhook/1.0")
            .body(payload_json.to_string())
            .send()
            .await
            .map_err(|e| WebhookError::RequestFailed(e.to_string()))?;

        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            Err(WebhookError::HttpError(status.as_u16()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("HTTP error with status: {0}")]
    HttpError(u16),

    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}