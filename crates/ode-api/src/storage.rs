use uuid::Uuid;
use tracing::info;
use std::error::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct S3Storage {
    bucket: String,
    mock_data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl S3Storage {
    pub async fn new(bucket: String) -> Result<Self, Box<dyn Error>> {
        info!("Storage service initialized with bucket: {} (mock mode)", bucket);

        Ok(Self {
            bucket,
            mock_data: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn store_html(&self, job_id: Uuid, content: Vec<u8>) -> Result<String, Box<dyn Error>> {
        let key = format!("jobs/{}/output.html", job_id);
        
        let mut data = self.mock_data.write().await;
        data.insert(key.clone(), content);
        
        let url = format!("https://{}.s3.amazonaws.com/{}", self.bucket, key);
        info!("Stored HTML for job {} at {}", job_id, url);

        Ok(url)
    }

    pub async fn store_pdf(&self, job_id: Uuid, content: Vec<u8>) -> Result<String, Box<dyn Error>> {
        let key = format!("jobs/{}/input.pdf", job_id);
        
        let mut data = self.mock_data.write().await;
        data.insert(key.clone(), content);
        
        let url = format!("https://{}.s3.amazonaws.com/{}", self.bucket, key);
        info!("Stored PDF for job {} at {}", job_id, url);

        Ok(url)
    }

    pub async fn delete_job_assets(&self, job_id: Uuid) -> Result<(), Box<dyn Error>> {
        let prefix = format!("jobs/{}/", job_id);
        let mut data = self.mock_data.write().await;
        
        let mut deleted_count = 0;
        data.retain(|k, _| {
            if k.starts_with(&prefix) {
                deleted_count += 1;
                info!("Deleted storage object: {}", k);
                false
            } else {
                true
            }
        });

        info!("Deleted {} assets for job {}", deleted_count, job_id);
        Ok(())
    }

    pub async fn get_file(&self, key: &str) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let data = self.mock_data.read().await;
        Ok(data.get(key).cloned())
    }
}