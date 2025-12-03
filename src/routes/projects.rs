use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::project::{self, Entity as Project};
use crate::middleware::auth::AuthUser;
use crate::error::AppError;
use crate::pagination::Pagination;
use axum::extract::Query;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateProjectRequest {
    name: String,
    description: Option<String>,
    #[schema(value_type = Object)]
    settings: Option<Value>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectRequest {
    name: Option<String>,
    description: Option<String>,
    #[schema(value_type = Object)]
    settings: Option<Value>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectResponse {
    #[schema(value_type = String)]
    id: Uuid,
    name: String,
    description: Option<String>,
    #[schema(value_type = Object)]
    settings: Value,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

impl From<project::Model> for ProjectResponse {
    fn from(project: project::Model) -> Self {
        ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            settings: project.settings,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

#[utoipa::path(
    post,
    path = "/projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created successfully", body = ProjectResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project Management"
)]
pub async fn create_project(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    println!("Create project request for user: {}", auth_user.username);

    let project = project::ActiveModel {
        id: Set(Uuid::new_v4()),
        owner_id: Set(auth_user.id),
        name: Set(payload.name),
        description: Set(payload.description),
        settings: Set(payload.settings.unwrap_or(serde_json::json!({}))),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };

    let created_project = project.insert(&db).await?;

    println!("Project '{}' created successfully", created_project.name);
    Ok((StatusCode::CREATED, Json(ProjectResponse::from(created_project))))
}

#[utoipa::path(
    get,
    path = "/projects",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "List of user's projects", body = [ProjectResponse]),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project Management"
)]
pub async fn list_projects(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<ProjectResponse>>, AppError> {
    println!("List projects request for user: {}", auth_user.username);

    // Filter by owner_id and deleted_at is null
    let projects = Project::find()
        .filter(project::Column::OwnerId.eq(auth_user.id))
        .filter(project::Column::DeletedAt.is_null())
        .limit(pagination.limit())
        .offset(pagination.offset())
        .all(&db)
        .await?;

    let responses: Vec<ProjectResponse> = projects.into_iter().map(ProjectResponse::from).collect();
    Ok(Json(responses))
}

#[utoipa::path(
    get,
    path = "/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project details", body = ProjectResponse),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project Management"
)]
pub async fn get_project(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, AppError> {
    println!("Get project request for ID: {}", project_id);

    let project = Project::find_by_id(project_id)
        .filter(project::Column::OwnerId.eq(auth_user.id))
        .filter(project::Column::DeletedAt.is_null())
        .one(&db)
        .await?
        .ok_or(AppError::NotFound("Project not found".to_string()))?;

    Ok(Json(ProjectResponse::from(project)))
}

#[utoipa::path(
    put,
    path = "/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated successfully", body = ProjectResponse),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project Management"
)]
pub async fn update_project(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    println!("Update project request for ID: {}", project_id);

    let project = Project::find_by_id(project_id)
        .filter(project::Column::OwnerId.eq(auth_user.id))
        .filter(project::Column::DeletedAt.is_null())
        .one(&db)
        .await?
        .ok_or(AppError::NotFound("Project not found".to_string()))?;

    let mut active_project = project.into_active_model();
    
    if let Some(name) = payload.name {
        active_project.name = Set(name);
    }
    if let Some(description) = payload.description {
        active_project.description = Set(Some(description));
    }
    if let Some(settings) = payload.settings {
        active_project.settings = Set(settings);
    }
    
    active_project.updated_at = Set(chrono::Utc::now().naive_utc());

    let updated_project = active_project.update(&db).await?;

    Ok(Json(ProjectResponse::from(updated_project)))
}

#[utoipa::path(
    delete,
    path = "/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project deleted successfully"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Project Management"
)]
pub async fn delete_project(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    println!("Delete project request for ID: {}", project_id);

    let project = Project::find_by_id(project_id)
        .filter(project::Column::OwnerId.eq(auth_user.id))
        .filter(project::Column::DeletedAt.is_null())
        .one(&db)
        .await?
        .ok_or(AppError::NotFound("Project not found".to_string()))?;

    let mut active_project = project.into_active_model();
    active_project.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));
    
    active_project.update(&db).await?;

    Ok(Json(serde_json::json!({
        "message": "Project deleted successfully"
    })))
}
