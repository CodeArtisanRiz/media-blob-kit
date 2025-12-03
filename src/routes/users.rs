use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QuerySelect, ModelTrait};
use serde::{Deserialize, Serialize};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use crate::entities::user::{self, Entity as User};
use crate::middleware::auth::AuthUser;
use uuid::Uuid;
use crate::pagination::Pagination;
use axum::extract::Query;
use crate::error::AppError;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    username: String,
    password: String,
    role: UserRole,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

impl From<UserRole> for user::Role {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Admin => user::Role::Admin,
            UserRole::User => user::Role::User,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    #[schema(value_type = String)]
    id: Uuid,
    username: String,
    role: user::Role,
    created_at: chrono::NaiveDateTime,
}

impl From<user::Model> for UserResponse {
    fn from(user: user::Model) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 409, description = "Username already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User Management"
)]
pub async fn create_user(
    State(db): State<DatabaseConnection>,
    axum::Extension(_auth_user): axum::Extension<AuthUser>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    println!("Create user request for: {}", payload.username);

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| {
            eprintln!("Password hash error: {}", e);
            AppError::InternalServerError("Password hashing failed".to_string())
        })?
        .to_string();

    // Create user
    let user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(payload.username),
        password: Set(password_hash),
        role: Set(payload.role.into()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };

    match user.insert(&db).await {
        Ok(created_user) => {
            println!("User '{}' created successfully", created_user.username);
            Ok((StatusCode::CREATED, Json(UserResponse::from(created_user))))
        }
        Err(e) => {
            eprintln!("Failed to create user: {}", e);
            if e.to_string().contains("duplicate key value violates unique constraint") {
                return Err(AppError::Conflict("Username already exists".to_string()));
            }
            Err(AppError::DatabaseError(e))
        }
    }
}

#[utoipa::path(
    get,
    path = "/users",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "List of all users", body = [UserResponse]),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User Management"
)]
pub async fn list_users(
    State(db): State<DatabaseConnection>,
    _auth_user: axum::Extension<AuthUser>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    println!("List users request");

    let users = User::find()
        .limit(pagination.limit())
        .offset(pagination.offset())
        .all(&db)
        .await?;

    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(user_responses))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "User ID to delete")
    ),
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 400, description = "Cannot delete yourself"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User Management"
)]
pub async fn delete_user(
    State(db): State<DatabaseConnection>,
    axum::Extension(auth_user): axum::Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    println!("Delete user request for ID: {}", user_id);

    // Prevent deleting self
    if auth_user.id == user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".to_string()));
    }

    let user = User::find_by_id(user_id)
        .one(&db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    user.delete(&db).await?;

    Ok(Json(serde_json::json!({
        "message": "User deleted successfully"
    })))
}
