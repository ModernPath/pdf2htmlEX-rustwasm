use redis::{AsyncCommands, aio::ConnectionManager};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

const QUEUE_NAME: &str = "ode:conversion:queue";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionTask {
    pub job_id: Uuid,
}

pub struct TaskQueue {
    conn: ConnectionManager,
}

impl TaskQueue {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    pub async fn enqueue_job(&mut self, job_id: Uuid) -> Result<(), redis::RedisError> {
        let task = ConversionTask { job_id };
        let task_json = serde_json::to_string(&task)
            .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;

        let _: () = self.conn.rpush(QUEUE_NAME, &task_json).await?;
        Ok(())
    }

    pub async fn dequeue_job(&mut self) -> Result<Option<ConversionTask>, redis::RedisError> {
        let result: Option<String> = self.conn.lpop(QUEUE_NAME, None).await?;

        if let Some(task_json) = result {
            let task: ConversionTask = serde_json::from_str(&task_json)
                .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    pub async fn queue_length(&mut self) -> Result<usize, redis::RedisError> {
        let len: usize = self.conn.llen(QUEUE_NAME).await?;
        Ok(len)
    }
}