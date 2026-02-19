use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;
use crate::models::{JobStatus, JobMetadata, ConversionProfile, ConversionOptions, CreateProfileRequest, UpdateProfileRequest, User, ApiKey, Role};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;

        // Run migrations (each statement executed separately)
        let migrations = [
            r#"CREATE TABLE IF NOT EXISTS jobs (
                id UUID PRIMARY KEY,
                status VARCHAR(50) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                file_name VARCHAR(255) NOT NULL,
                file_size BIGINT NOT NULL,
                webhook_url TEXT,
                error_message TEXT,
                pdf_data BYTEA NOT NULL,
                config JSONB NOT NULL,
                result_url TEXT,
                profile_id UUID
            )"#,
            "CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status)",
            "CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_jobs_profile_id ON jobs(profile_id)",
            r#"CREATE TABLE IF NOT EXISTS conversion_profiles (
                id UUID PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                config JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL
            )"#,
            "CREATE INDEX IF NOT EXISTS idx_conversion_profiles_name ON conversion_profiles(name)",
            r#"CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                role VARCHAR(50) NOT NULL DEFAULT 'Developer',
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL
            )"#,
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
            "CREATE INDEX IF NOT EXISTS idx_users_role ON users(role)",
            r#"CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                key_hash VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT true,
                last_used_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL,
                expires_at TIMESTAMPTZ
            )"#,
            "CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id)",
            "CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash)",
        ];

        for migration in &migrations {
            sqlx::query(migration).execute(&pool).await?;
        }

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn create_job(
        &self,
        id: Uuid,
        file_name: String,
        file_size: u64,
        pdf_data: &[u8],
        config: serde_json::Value,
        webhook_url: Option<String>,
        profile_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO jobs (id, status, created_at, updated_at, file_name, file_size, webhook_url, pdf_data, config, profile_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(id)
        .bind(JobStatus::Pending.to_string())
        .bind(now)
        .bind(now)
        .bind(&file_name)
        .bind(file_size as i64)
        .bind(&webhook_url)
        .bind(pdf_data)
        .bind(&config)
        .bind(profile_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_job(&self, id: Uuid) -> Result<Option<JobMetadata>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, chrono::DateTime<Utc>, chrono::DateTime<Utc>, String, i64, Option<String>, Option<String>, Option<Uuid>, Option<String>)>(
            "SELECT id, status, created_at, updated_at, file_name, file_size, webhook_url, error_message, profile_id, result_url FROM jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, status, created_at, updated_at, file_name, file_size, webhook_url, error_message, profile_id, result_url)) = row {
            let status = match status.as_str() {
                "pending" => JobStatus::Pending,
                "processing" => JobStatus::Processing,
                "completed" => JobStatus::Completed,
                "failed" => JobStatus::Failed,
                _ => return Err(sqlx::Error::RowNotFound),
            };

            Ok(Some(JobMetadata {
                id,
                status,
                created_at,
                updated_at,
                file_name,
                file_size: file_size as u64,
                webhook_url,
                error_message,
                profile_id,
                result_url,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_job_status(
        &self,
        id: Uuid,
        status: JobStatus,
        error_message: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE jobs SET status = $1, updated_at = $2, error_message = $3
            WHERE id = $4
            "#
        )
        .bind(status.to_string())
        .bind(now)
        .bind(&error_message)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_job_pdf_data(&self, id: Uuid) -> Result<Option<Vec<u8>>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Vec<u8>,)>(
            "SELECT pdf_data FROM jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(data,)| data))
    }

    pub async fn get_job_config(&self, id: Uuid) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let row = sqlx::query_as::<_, (serde_json::Value,)>(
            "SELECT config FROM jobs WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(config,)| config))
    }

    pub async fn set_result_url(&self, id: Uuid, result_url: String) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE jobs SET result_url = $1, updated_at = $2
            WHERE id = $3
            "#
        )
        .bind(&result_url)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_job(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM jobs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_profile(&self, request: CreateProfileRequest) -> Result<ConversionProfile, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let config_json = serde_json::to_value(&request.config)
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        sqlx::query(
            r#"
            INSERT INTO conversion_profiles (id, name, description, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&config_json)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ConversionProfile {
            id,
            name: request.name,
            description: request.description,
            config: request.config,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_profile(&self, id: Uuid) -> Result<Option<ConversionProfile>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, Option<String>, serde_json::Value, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, name, description, config, created_at, updated_at FROM conversion_profiles WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, name, description, config_json, created_at, updated_at)) = row {
            let config: ConversionOptions = serde_json::from_value(config_json)
                .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

            Ok(Some(ConversionProfile {
                id,
                name,
                description,
                config,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_profiles(&self) -> Result<Vec<ConversionProfile>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, Option<String>, serde_json::Value, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, name, description, config, created_at, updated_at FROM conversion_profiles ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut profiles = Vec::new();
        for (id, name, description, config_json, created_at, updated_at) in rows {
            let config: ConversionOptions = serde_json::from_value(config_json)
                .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

            profiles.push(ConversionProfile {
                id,
                name,
                description,
                config,
                created_at,
                updated_at,
            });
        }

        Ok(profiles)
    }

    pub async fn update_profile(&self, id: Uuid, request: UpdateProfileRequest) -> Result<Option<ConversionProfile>, sqlx::Error> {
        let now = Utc::now();

        let mut updates = Vec::new();
        let mut bind_count = 2;

        if request.name.is_some() {
            updates.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }
        if request.description.is_some() {
            updates.push(format!("description = ${}", bind_count));
            bind_count += 1;
        }
        if request.config.is_some() {
            updates.push(format!("config = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            return self.get_profile(id).await;
        }

        updates.push(format!("updated_at = ${}", bind_count));
        bind_count += 1;

        let query_str = format!(
            "UPDATE conversion_profiles SET {} WHERE id = $1 RETURNING id, name, description, config, created_at, updated_at",
            updates.join(", ")
        );

        let mut query = sqlx::query_as::<_, (Uuid, String, Option<String>, serde_json::Value, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(&query_str);
        query = query.bind(id);

        if let Some(name) = request.name {
            query = query.bind(name);
        }
        if let Some(description) = request.description {
            query = query.bind(description);
        }
        if let Some(config) = request.config {
            let config_json = serde_json::to_value(&config)
                .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
            query = query.bind(config_json);
        }
        query = query.bind(now);

        let row = query.fetch_optional(&self.pool).await?;

        if let Some((id, name, description, config_json, created_at, updated_at)) = row {
            let config: ConversionOptions = serde_json::from_value(config_json)
                .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

            Ok(Some(ConversionProfile {
                id,
                name,
                description,
                config,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_profile(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM conversion_profiles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_user(&self, email: String, password_hash: String, role: Role) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, email, role, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&email)
        .bind(&password_hash)
        .bind(role.to_string())
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map(|row| User {
            id: row.try_get("id").unwrap_or_default(),
            email: row.try_get("email").unwrap_or_default(),
            password_hash: String::new(),
            role: Role::from_str(row.try_get("role").unwrap_or("Viewer".to_string()).as_str()),
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
        })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, String, String, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, email, password_hash, role, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, email, password_hash, role, created_at, updated_at)| User {
            id,
            email,
            password_hash,
            role: Role::from_str(role.as_str()),
            created_at,
            updated_at,
        }))
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, String, String, String, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, email, password_hash, role, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, email, password_hash, role, created_at, updated_at)| User {
            id,
            email,
            password_hash,
            role: Role::from_str(role.as_str()),
            created_at,
            updated_at,
        }))
    }

    pub async fn create_api_key(&self, user_id: Uuid, key_hash: String, name: String) -> Result<ApiKey, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO api_keys (id, user_id, key_hash, name, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, name, is_active, last_used_at, created_at, expires_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(&key_hash)
        .bind(&name)
        .bind(true)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map(|row| ApiKey {
            id: row.try_get("id").unwrap_or_default(),
            user_id: row.try_get("user_id").unwrap_or_default(),
            key_hash: String::new(),
            name: row.try_get("name").unwrap_or_default(),
            is_active: row.try_get("is_active").unwrap_or(true),
            last_used_at: row.try_get("last_used_at").ok(),
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            expires_at: row.try_get("expires_at").ok(),
        })
    }

    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, bool, Option<chrono::DateTime<Utc>>, chrono::DateTime<Utc>, Option<chrono::DateTime<Utc>>)>(
            "SELECT id, user_id, key_hash, name, is_active, last_used_at, created_at, expires_at FROM api_keys WHERE key_hash = $1 AND is_active = true"
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, user_id, key_hash, name, is_active, last_used_at, created_at, expires_at)| ApiKey {
            id,
            user_id,
            key_hash,
            name,
            is_active,
            last_used_at,
            created_at,
            expires_at,
        }))
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, bool, Option<chrono::DateTime<Utc>>, chrono::DateTime<Utc>, Option<chrono::DateTime<Utc>>)>(
            "SELECT id, user_id, key_hash, name, is_active, last_used_at, created_at, expires_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|(id, user_id, key_hash, name, is_active, last_used_at, created_at, expires_at)| ApiKey {
            id: *id,
            user_id: *user_id,
            key_hash: key_hash.clone(),
            name: name.clone(),
            is_active: *is_active,
            last_used_at: *last_used_at,
            created_at: *created_at,
            expires_at: *expires_at,
        }).collect())
    }

    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_api_key_last_used(&self, key_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET last_used_at = $1 WHERE id = $2")
            .bind(Utc::now())
            .bind(key_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_all_active_api_keys(&self) -> Result<Vec<(Uuid, String, String)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, String)>(
            "SELECT id, key_hash, name FROM api_keys WHERE is_active = true ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_system_stats(&self) -> Result<(i64, i64, i64, i64), sqlx::Error> {
        let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        let total_api_keys: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE is_active = true")
            .fetch_one(&self.pool)
            .await?;

        let total_jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs")
            .fetch_one(&self.pool)
            .await?;

        let processing_jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status = 'processing'")
            .fetch_one(&self.pool)
            .await?;

        Ok((total_users, total_api_keys, total_jobs, processing_jobs))
    }
}