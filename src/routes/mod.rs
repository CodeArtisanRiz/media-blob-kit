mod home;
mod auth;

use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::DatabaseConnection;

pub fn create_routes(db: DatabaseConnection) -> Router {
    Router::new()
        .route("/", get(home::root))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .with_state(db)
}

