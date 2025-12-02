use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use rand::{RngCore, thread_rng};
use base64::{Engine as _, engine::general_purpose};

use crate::entities::{api_key, project};
use crate::middleware::auth::AuthUser;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateApiKeyRequest {
    name: String,
    expires_at: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateApiKeyRequest {
    is_active: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ApiKeyResponse {
    #[schema(value_type = String)]
    id: Uuid,
    name: String,
    created_at: chrono::NaiveDateTime,
    expires_at: Option<chrono::NaiveDateTime>,
    is_active: bool,
    // Only returned on creation
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
}

impl From<api_key::Model> for ApiKeyResponse {
    fn from(model: api_key::Model) -> Self {
        ApiKeyResponse {
            id: model.id,
            name: model.name,
            created_at: model.created_at,
            expires_at: model.expires_at,
            is_active: model.is_active,
            key: None,
        }
    }
}

#[utoipa::path(
    post,
    path = "/projects/{id}/keys",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API Key created successfully", body = ApiKeyResponse),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project API Keys"
)]
pub async fn create_api_key(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, StatusCode> {
    println!("Create API key request for project: {}", project_id);

    // Verify project ownership
    let user = crate::entities::user::Entity::find()
        .filter(crate::entities::user::Column::Username.eq(&auth_user.username))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let project = project::Entity::find_by_id(project_id)
        .filter(project::Column::OwnerId.eq(user.id))
        .filter(project::Column::DeletedAt.is_null())
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Generate API Key
    let mut key_bytes = [0u8; 32];
    thread_rng().fill_bytes(&mut key_bytes);
    let raw_key = format!("mbk_{}", general_purpose::URL_SAFE_NO_PAD.encode(key_bytes));

    // Hash API Key
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    let api_key = api_key::ActiveModel {
        id: Set(Uuid::new_v4()),
        project_id: Set(project.id),
        name: Set(payload.name),
        key_hash: Set(key_hash),
        created_at: Set(chrono::Utc::now().naive_utc()),
        expires_at: Set(payload.expires_at),
        is_active: Set(true),
    };

    match api_key.insert(&db).await {
        Ok(created_key) => {
            let mut response = ApiKeyResponse::from(created_key);
            response.key = Some(raw_key);
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Failed to create API key: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/projects/{id}/keys",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "List of API Keys", body = [ApiKeyResponse]),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project API Keys"
)]
pub async fn list_api_keys(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ApiKeyResponse>>, StatusCode> {
    println!("List API keys request for project: {}", project_id);

    // Verify project ownership
    let user = crate::entities::user::Entity::find()
        .filter(crate::entities::user::Column::Username.eq(&auth_user.username))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let project = project::Entity::find_by_id(project_id)
        .filter(project::Column::OwnerId.eq(user.id))
        .filter(project::Column::DeletedAt.is_null())
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let keys = api_key::Entity::find()
        .filter(api_key::Column::ProjectId.eq(project.id))
        .all(&db)
        .await
        .map_err(|e| {
            eprintln!("Failed to list API keys: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let responses: Vec<ApiKeyResponse> = keys.into_iter().map(ApiKeyResponse::from).collect();
    Ok(Json(responses))
}

#[utoipa::path(
    patch,
    path = "/projects/{id}/keys/{key_id}",
    params(
        ("id" = String, Path, description = "Project ID"),
        ("key_id" = String, Path, description = "API Key ID")
    ),
    request_body = UpdateApiKeyRequest,
    responses(
        (status = 200, description = "API Key updated successfully"),
        (status = 404, description = "Project or API Key not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project API Keys"
)]
pub async fn update_api_key(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path((project_id, key_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("Update API key request for project: {}, key: {}", project_id, key_id);

    // Verify project ownership
    let user = crate::entities::user::Entity::find()
        .filter(crate::entities::user::Column::Username.eq(&auth_user.username))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let project = project::Entity::find_by_id(project_id)
        .filter(project::Column::OwnerId.eq(user.id))
        .filter(project::Column::DeletedAt.is_null())
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let key = api_key::Entity::find_by_id(key_id)
        .filter(api_key::Column::ProjectId.eq(project.id))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active_key = key.into_active_model();
    active_key.is_active = Set(payload.is_active);

    active_key.update(&db).await.map_err(|e| {
        eprintln!("Failed to update API key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "message": "API Key updated successfully"
    })))
}

#[utoipa::path(
    delete,
    path = "/projects/{id}/keys/{key_id}",
    params(
        ("id" = String, Path, description = "Project ID"),
        ("key_id" = String, Path, description = "API Key ID")
    ),
    responses(
        (status = 200, description = "API Key deleted successfully"),
        (status = 404, description = "Project or API Key not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project API Keys"
)]
pub async fn delete_api_key(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path((project_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("Delete API key request for project: {}, key: {}", project_id, key_id);

    // Verify project ownership
    let user = crate::entities::user::Entity::find()
        .filter(crate::entities::user::Column::Username.eq(&auth_user.username))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let project = project::Entity::find_by_id(project_id)
        .filter(project::Column::OwnerId.eq(user.id))
        .filter(project::Column::DeletedAt.is_null())
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let key = api_key::Entity::find_by_id(key_id)
        .filter(api_key::Column::ProjectId.eq(project.id))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    api_key::Entity::delete(key.into_active_model()).exec(&db).await.map_err(|e| {
        eprintln!("Failed to delete API key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "message": "API Key deleted successfully"
    })))
}
