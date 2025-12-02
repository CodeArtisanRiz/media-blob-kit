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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Define the OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        // General endpoints
        home::root,
        // Authentication endpoints
        auth::login,
        auth::refresh,
        auth::logout,
        auth::me,
        // User management endpoints
        users::create_user,
        users::list_users,
        users::delete_user,
    ),
    components(
        schemas(
            // Home schemas
            home::RootResponse,
            // Auth schemas
            auth::LoginRequest,
            auth::LoginResponse,
            auth::RefreshRequest,
            auth::RefreshResponse,
            auth::LogoutRequest,
            auth::LogoutResponse,
            auth::ErrorResponse,
            auth::UserProfile,
            // User schemas
            users::CreateUserRequest,
            users::UserResponse,
            users::UserRole,
            crate::entities::user::Role,
        )
    ),
    tags(
        (name = "General", description = "General API information"),
        (name = "Authentication", description = "Authentication endpoints for login, token refresh, and logout"),
        (name = "User Management", description = "User management endpoints (superuser access required)")
    ),
    info(
        title = "MediaBlobKit API",
        version = "0.1.0",
        description = "A Rust/Axum application for media blob management with user authentication and role-based access control",
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

// Add security scheme for JWT Bearer tokens
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::Http::new(
                    utoipa::openapi::security::HttpAuthScheme::Bearer
                )
            ),
        );
    }
}

pub fn create_routes(db: DatabaseConnection) -> Router {
    // Swagger UI (stateless)  
    let swagger_router: Router = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .into();

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

    // Public routes (no auth required) and merge all together
    let app_routes = Router::new()
        .route("/", get(home::root))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .merge(protected_routes)
        .merge(su_routes)
        .with_state(db);
    
    // Merge Swagger UI (which has no state) with the rest
    Router::new()
        .merge(swagger_router)
        .merge(app_routes)
}
