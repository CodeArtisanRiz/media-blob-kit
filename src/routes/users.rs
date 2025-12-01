use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use crate::entities::user::{self, Entity as User};
use crate::middleware::auth::AuthUser;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    password: String,
    role: UserRole,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum UserRole {
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

#[derive(Serialize)]
pub struct UserResponse {
    id: i32,
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

pub async fn create_user(
    State(db): State<DatabaseConnection>,
    _auth_user: axum::Extension<AuthUser>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    println!("Create user request for: {}", payload.username);

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| {
            eprintln!("Password hash error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_string();

    // Create user
    let user = user::ActiveModel {
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
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_users(
    State(db): State<DatabaseConnection>,
    _auth_user: axum::Extension<AuthUser>,
) -> impl IntoResponse {
    println!("List users request");

    match User::find().all(&db).await {
        Ok(users) => {
            let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            Ok(Json(user_responses))
        }
        Err(e) => {
            eprintln!("Failed to list users: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_user(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<AuthUser>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    println!("Delete user request for ID: {}", user_id);

    // Find user to delete
    let user_to_delete = User::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match user_to_delete {
        Some(user) => {
            // Prevent su from deleting themselves
            if user.username == auth_user.username {
                eprintln!("User tried to delete themselves");
                return Err(StatusCode::BAD_REQUEST);
            }

            // Delete user
            let active_user: user::ActiveModel = user.into();
            active_user.delete(&db).await.map_err(|e| {
                eprintln!("Delete error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            println!("User ID {} deleted successfully", user_id);
            Ok(Json(serde_json::json!({
                "message": "User deleted successfully"
            })))
        }
        None => {
            eprintln!("User ID {} not found", user_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
