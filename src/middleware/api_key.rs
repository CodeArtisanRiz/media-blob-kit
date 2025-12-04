use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use crate::entities::api_key::{self, Entity as ApiKey};
use crate::entities::project::Entity as Project;
use crate::error::AppError;

use crate::models::settings::ProjectSettings;

#[derive(Clone, Debug)]
pub struct ProjectContext {
    pub id: uuid::Uuid,
    pub name: String,
    pub settings: ProjectSettings,
}

pub async fn api_key_auth(
    axum::extract::State(db): axum::extract::State<DatabaseConnection>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key_header = headers
        .get("x-api-key")
        .ok_or(AppError::Unauthorized("Missing API Key".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid API Key format".to_string()))?;

    let mut hasher = Sha256::new();
    hasher.update(api_key_header.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    // Find API Key and related Project
    let (api_key, project) = ApiKey::find()
        .filter(api_key::Column::KeyHash.eq(&key_hash))
        .find_also_related(Project)
        .one(&db)
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::Unauthorized("Invalid API Key".to_string()))?;

    let project = project.ok_or(AppError::InternalServerError("Orphaned API Key".to_string()))?;

    if !api_key.is_active {
        return Err(AppError::Unauthorized("API Key is inactive".to_string()));
    }

    if let Some(expires_at) = api_key.expires_at {
        if expires_at < chrono::Utc::now().naive_utc() {
            return Err(AppError::Unauthorized("API Key has expired".to_string()));
        }
    }

    println!("Raw Project Settings: {:?}", project.settings);
    let settings: ProjectSettings = serde_json::from_value(project.settings.clone())
        .map_err(|e| {
            eprintln!("Failed to parse project settings: {}", e);
            e
        })
        .unwrap_or_default();
    println!("Parsed Settings: {:?}", settings);

    request.extensions_mut().insert(ProjectContext {
        id: project.id,
        name: project.name,
        settings,
    });

    Ok(next.run(request).await)
}
