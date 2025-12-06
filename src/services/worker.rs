use std::time::Duration;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, 
    QueryOrder, QuerySelect, Set, TransactionTrait, ConnectionTrait
};
use sea_orm::sea_query::{LockType, LockBehavior};
use tokio::time::sleep;
use crate::entities::{job, file, project};
use crate::services::s3::S3Service;
use crate::utils::{image_processor, sanitize_bucket_name};
use crate::models::settings::VariantConfig;
use std::collections::HashMap;

pub struct Worker {
    db: DatabaseConnection,
    s3: S3Service,
}

fn format_size(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    
    if bytes as f64 >= MB {
        format!("{:.2}MiB", bytes as f64 / MB)
    } else if bytes as f64 >= KB {
        format!("{:.2}kb", bytes as f64 / KB)
    } else {
        format!("{}b", bytes)
    }
}

impl Worker {
    pub async fn new(db: DatabaseConnection) -> Self {
        let s3 = S3Service::new().await;
        Self { db, s3 }
    }

    pub async fn run(&self) {
        println!("Worker started");
        
        // Recover any jobs stuck in 'processing' state from previous runs
        if let Err(e) = self.recover_stuck_jobs().await {
            eprintln!("Failed to recover stuck jobs: {}", e);
        }

        loop {
            if let Err(e) = self.process_next_job().await {
                eprintln!("Worker error: {}", e);
            }
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn recover_stuck_jobs(&self) -> Result<(), String> {
        // Reset any jobs that are 'processing' back to 'pending'
        // In a single-worker environment, this is safe on startup.
        // In a multi-worker environment, this would need a timeout check/heartbeat.
        
        let parse_result = sea_orm::Statement::from_string(
            self.db.get_database_backend(),
            "UPDATE jobs SET status = 'pending' WHERE status = 'processing'".to_owned(),
        );

        let result = self.db.execute(parse_result).await.map_err(|e| e.to_string())?;
        
        if result.rows_affected() > 0 {
            println!("Recovered {} stuck jobs (reset to pending)", result.rows_affected());
        }

        Ok(())
    }

    async fn process_next_job(&self) -> Result<(), String> {
        // Start transaction
        let txn = self.db.begin().await.map_err(|e| e.to_string())?;

        // 1. Find pending job with lock
        // Note: SeaORM 1.0+ supports locking. 
        // We use raw query or specific lock methods if available.
        // For now, we'll try standard find with lock.
        
        let job_opt = job::Entity::find()
            .filter(job::Column::Status.eq("pending"))
            .order_by_asc(job::Column::CreatedAt)
            .limit(1)
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?;

        let job_model = match job_opt {
            Some(j) => j,
            None => return Ok(()), // No jobs
        };

        println!("Worker picked up job {}", job_model.id);

        // Update job status to processing
        let mut job_active: job::ActiveModel = job_model.clone().into();
        job_active.status = Set("processing".to_string());
        job_active.updated_at = Set(chrono::Utc::now().naive_utc());
        let job_model = job_active.update(&txn).await.map_err(|e| e.to_string())?;

        // Commit transaction to release lock and save 'processing' state
        txn.commit().await.map_err(|e| e.to_string())?;

        // Now process the job (outside transaction to avoid holding DB lock during S3 ops)
        // We re-fetch related data as needed.
        let job_start_time = std::time::Instant::now();
        
        match self.handle_job(&job_model).await {
            Ok(_) => {
                let duration = job_start_time.elapsed();
                println!("Job {} completed successfully took {:.2?}", job_model.id, duration);
                let mut job_active: job::ActiveModel = job_model.into();
                job_active.status = Set("completed".to_string());
                job_active.updated_at = Set(chrono::Utc::now().naive_utc());
                job_active.update(&self.db).await.map_err(|e| e.to_string())?;
            },
            Err(e) => {
                eprintln!("Job {} failed: {}", job_model.id, e);
                let payload = job_model.payload.clone();
                let mut job_active: job::ActiveModel = job_model.into();
                job_active.status = Set("failed".to_string());
                job_active.payload = Set(serde_json::json!({
                    "error": e,
                    "original_payload": payload
                }));
                job_active.updated_at = Set(chrono::Utc::now().naive_utc());
                job_active.update(&self.db).await.map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    async fn handle_job(&self, job: &job::Model) -> Result<(), String> {
        // 1. Get File and Project
        let file = file::Entity::find_by_id(job.file_id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("File not found")?;

        let project = project::Entity::find_by_id(file.project_id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Project not found")?;

        // 2. Parse payload for variants
        let payload = job.payload.as_object().ok_or("Invalid payload")?;
        let variants_json = payload.get("variants").ok_or("No variants in payload")?;
        let variants: HashMap<String, VariantConfig> = serde_json::from_value(variants_json.clone())
            .map_err(|e| e.to_string())?;

        // 3. Download original file
        let original_data = self.s3.get_object(&file.s3_key).await.map_err(|e| e.to_string())?;

        // 4. Process each variant
        // 4. Process each variant
        for (variant_name, config) in variants {
            println!("Processing variant: {}", variant_name);
            let start_time = std::time::Instant::now();
            
            // Clone data to move into validation closure
            let original_data_clone = original_data.clone();
            let config_clone = config.clone();

            // Process image in blocking thread
            let (processed_data, mime_type) = tokio::task::spawn_blocking(move || {
                image_processor::process_image(&original_data_clone, &config_clone)
            }).await
              .map_err(|e| format!("Task join error: {}", e))?
              .map_err(|e| e.to_string())?;

            let elapsed = start_time.elapsed();
            let src_size = format_size(original_data.len());
            let dest_size = format_size(processed_data.len());
            println!("took {:.2?} | {} -> {}", elapsed, src_size, dest_size);

            // Determine extension
            let ext = match mime_type.as_str() {
                "image/avif" => "avif",
                "image/webp" => "webp",
                "image/png" => "png",
                "image/jpeg" => "jpg",
                _ => "bin",
            };

            // Construct S3 Key
            // Format: {project_name}-{project_id}/images/{variant_name}/{file_id}.{ext}
            let s3_key = format!("{}-{}/images/{}/{}.{}", 
                sanitize_bucket_name(&project.name), 
                project.id, 
                variant_name, 
                file.id, 
                ext
            );

            // Upload to S3
            println!("Uploading variant {} to S3 key: {}", variant_name, s3_key);
            self.s3.put_object(&s3_key, processed_data, &mime_type).await.map_err(|e| e.to_string())?;
            println!("Variant {} uploaded successfully", variant_name);
        }

        // 5. Update File status
        let mut file_active: file::ActiveModel = file.into();
        file_active.status = Set("ready".to_string());
        file_active.updated_at = Set(chrono::Utc::now().naive_utc());
        file_active.update(&self.db).await.map_err(|e| e.to_string())?;

        Ok(())
    }
}
