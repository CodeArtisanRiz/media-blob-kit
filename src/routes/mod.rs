mod home;
mod auth;
mod users;

use axum::{
    routing::{get, post, delete},
    Router,
    middleware,
};
use sea_orm::DatabaseConnection;
use crate::middleware::auth::auth_middleware;
use crate::middleware::role::require_su;

pub fn create_routes(db: DatabaseConnection) -> Router {
    // Protected routes that require auth
    let protected_routes = Router::new()
        .route("/auth/me", get(auth::me))
        .layer(middleware::from_fn(auth_middleware));

    // Su-only routes
    let su_routes = Router::new()
        .route("/users", post(users::create_user))
        .route("/users", get(users::list_users))
        .route("/users/{id}", delete(users::delete_user))
        .layer(middleware::from_fn(require_su))
        .layer(middleware::from_fn(auth_middleware));

    // Public routes (no auth required)
    Router::new()
        .route("/", get(home::root))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .merge(protected_routes)
        .merge(su_routes)
        .with_state(db)
}
