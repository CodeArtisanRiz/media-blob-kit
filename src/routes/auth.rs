use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::entities::{user::{self, Entity as User}, refresh_token::{self, Entity as RefreshToken}};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use std::env;
use uuid::Uuid;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: usize,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RefreshResponse {
    access_token: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LogoutRequest {
    refresh_token: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct LogoutResponse {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    role: user::Role,
}

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!("WARNING: JWT_SECRET not set in .env, using default (insecure!)");
        "secret".to_string()
    })
}

fn generate_refresh_token() -> String {
    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut random_bytes);
    general_purpose::STANDARD.encode(random_bytes)
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    println!("Login attempt for: {}", payload.username);
    let user = User::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&db)
        .await
        .map_err(|e| {
            println!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(user) = user {
        println!("User found: {}", user.username);
        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| {
                println!("Hash Parse Error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        if Argon2::default()
            .verify_password(payload.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            println!("Password verified");
            
            // Generate access token (15 minutes)
            let expiration = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
                + 900; // 15 minutes

            let claims = Claims {
                sub: user.username.clone(),
                exp: expiration,
                role: user.role.clone(),
            };

            let access_token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(get_jwt_secret().as_ref()),
            )
            .map_err(|e| {
                println!("Token Encode Error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Generate refresh token (1 day)
            let refresh_token_str = generate_refresh_token();
            let token_hash = hash_token(&refresh_token_str);
            
            let refresh_expires_at = chrono::Utc::now().naive_utc() 
                + chrono::Duration::days(1);

            let refresh_token_model = refresh_token::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user.id),
                token_hash: Set(token_hash),
                expires_at: Set(refresh_expires_at),
                created_at: Set(chrono::Utc::now().naive_utc()),
                revoked: Set(false),
                ..Default::default()
            };

            refresh_token_model.insert(&db).await.map_err(|e| {
                println!("Refresh Token DB Error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            println!("Tokens generated successfully");
            return Ok(Json(LoginResponse { 
                access_token,
                refresh_token: refresh_token_str,
                expires_in: 900,
            }));
        } else {
            println!("Password verification failed");
        }
    } else {
        println!("User not found");
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn refresh(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    println!("Refresh token request");
    
    let token_hash = hash_token(&payload.refresh_token);
    
    // Find refresh token in database
    let refresh_token = RefreshToken::find()
        .filter(refresh_token::Column::TokenHash.eq(&token_hash))
        .one(&db)
        .await
        .map_err(|e| {
            println!("DB Error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Internal server error".to_string()
            }))
        })?;

    if let Some(token) = refresh_token {
        // Check if token is revoked
        if token.revoked {
            println!("Token is revoked");
            return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "User logged out. Please re-login.".to_string()
            })));
        }

        // Check if token is expired
        let now = chrono::Utc::now().naive_utc();
        if token.expires_at < now {
            println!("Token is expired");
            return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Refresh token expired. Please re-login.".to_string()
            })));
        }

        // Get user details
        let user = User::find_by_id(token.user_id)
            .one(&db)
            .await
            .map_err(|e| {
                println!("User lookup error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: "Internal server error".to_string()
                }))
            })?
            .ok_or((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "User not found. Please re-login.".to_string()
            })))?;

        // Generate new access token
        let expiration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + 900; // 15 minutes

        let claims = Claims {
            sub: user.username,
            exp: expiration,
            role: user.role,
        };

        let access_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(get_jwt_secret().as_ref()),
        )
        .map_err(|e| {
            println!("Token Encode Error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Failed to generate token".to_string()
            }))
        })?;

        println!("New access token generated");
        return Ok(Json(RefreshResponse { access_token }));
    }

    println!("Invalid refresh token");
    Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
        error: "Invalid refresh token. Please re-login.".to_string()
    })))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out successfully", body = LogoutResponse),
        (status = 404, description = "Refresh token not found")
    ),
    tag = "Authentication"
)]
pub async fn logout(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    println!("Logout request");
    
    let token_hash = hash_token(&payload.refresh_token);
    
    // Find and revoke refresh token
    let refresh_token = RefreshToken::find()
        .filter(refresh_token::Column::TokenHash.eq(&token_hash))
        .one(&db)
        .await
        .map_err(|e| {
            println!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(token) = refresh_token {
        let mut active_token: refresh_token::ActiveModel = token.into();
        active_token.revoked = Set(true);
        
        active_token.update(&db).await.map_err(|e| {
            println!("Update Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        println!("Refresh token revoked");
        return Ok(Json(LogoutResponse {
            message: "Logged out successfully".to_string(),
        }));
    }

    println!("Token not found");
    Err(StatusCode::NOT_FOUND)
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserProfile {
    #[schema(value_type = String)]
    id: Uuid,
    username: String,
    role: user::Role,
    created_at: chrono::NaiveDateTime,
}

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = UserProfile),
        (status = 401, description = "Unauthorized - Invalid or missing token")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn me(
    State(db): State<DatabaseConnection>,
    auth_user: axum::Extension<crate::middleware::auth::AuthUser>,
) -> impl IntoResponse {
    println!("/auth/me request for: {}", auth_user.username);
    
    // Find user in database
    let user = User::find()
        .filter(user::Column::Username.eq(&auth_user.username))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match user {
        Some(user) => {
            let profile = UserProfile {
                id: user.id,
                username: user.username,
                role: user.role,
                created_at: user.created_at,
            };
            Ok(Json(profile))
        }
        None => {
            eprintln!("User not found in database: {}", auth_user.username);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
