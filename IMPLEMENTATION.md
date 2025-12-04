# MediaBlobKit Implementation Roadmap

This document outlines the step-by-step implementation plan for the MediaBlobKit project.

## Phase 1: Foundation & Infrastructure Setup
**Goal**: Set up the core application structure, database connection, and configuration.

- [x] **Project Configuration**
    - [x] Setup `dotenv` for environment variables (DB_URL, JWT_SECRET, AWS_ACCESS_KEY, etc.).
    - [x] Create a `Config` struct to load and validate env vars.
- [x] **Database Setup**
    - [x] Choose an ORM/Query Builder (e.g., `sqlx` or `sea-orm`).
    - [x] Setup PostgreSQL database.
    - [x] Create initial migration for `users` table.
- [ ] **Logging & Tracing**
    - [ ] Configure `tracing-subscriber` for structured logging.

## Phase 2: Authentication & User Management
**Goal**: Implement secure user login and user creation by superusers only.

- [x] **User Models & Migrations**
    - [x] Define `User` struct with role enum (su, admin, user).
    - [x] Create migration for `users` (id, username, password, role, created_at).
- [x] **Auth Logic**
    - [x] Implement password hashing using `argon2`.
    - [x] Implement JWT token generation and validation.
    - [x] Implement refresh token system.
- [x] **Auth Endpoints**
    - [x] `POST /auth/login`: Validate credentials and return access + refresh tokens.
    - [x] `POST /auth/refresh`: Get new access token using refresh token.
    - [x] `POST /auth/logout`: Revoke refresh token.
    - [x] `GET /auth/me`: Get current user profile (protected).
- [x] **User Management** (Su-only)
    - [x] `POST /users`: Create new user (admin or user role) - requires su authentication.
    - [x] `GET /users`: List all users - requires su authentication.
    - [x] `DELETE /users/{id}`: Delete user - requires su authentication.
- [x] **Middleware**
    - [x] Create an `AuthMiddleware` to validate JWT and inject `UserId` into extensions.
    - [x] Create role-based authorization middleware.

**Note**: Superuser (su) accounts can only be created via CLI command `create-superuser` (requires shell access).

**JWT Token Limitation**: Access tokens are stateless JWTs with 15-minute expiration. When a user logs out, the refresh token is immediately revoked, but the access token remains valid until expiration (up to 15 minutes). This is standard JWT behavior. For immediate token revocation, a Redis-based token blacklist can be implemented in the future.


## Phase 3: Project Management
**Goal**: Allow users to create and manage projects with specific settings.

- [x] **Project Models & Migrations**
    - [x] Create migration for `projects` table (id, owner_id, name, settings_json, etc.).
- [x] **Project API**
    - [x] `POST /projects`: Create a new project.
    - [x] `GET /projects`: List user's projects.
    - [x] `GET /projects/:id`: Get project details.
    - [x] `PUT /projects/:id`: Update project settings (image variants, quotas).
    - [x] `DELETE /projects/:id`: Soft delete project.
- [x] **Project API Keys**
    - [x] `POST /projects/:id/keys`: Create a new API key for the project.
    - [x] `GET /projects/:id/keys`: List API keys for the project.
    - [x] `PATCH /projects/:id/keys/:key_id`: Enable/Disable an API key.
    - [x] `DELETE /projects/:id/keys/:key_id`: Permanently delete an API key.

> [!NOTE]
> **Future Improvement**: When updating project settings via `PUT /projects/:id`, if the `image_variants` configuration changes, we need to trigger a background job to:
> 1. Generate missing variants for existing images (if a new variant is added).
> 2. Delete obsolete variants (if a variant is removed).

## Phase 4: Core Improvements & Refactoring
**Goal**: Enhance code quality, performance, and standard features.

- [x] **Pagination**
    - [x] Create reusable `Pagination` and `PaginatedResponse` structs.
    - [x] Implement pagination for `list_projects`, `list_api_keys`, and `list_users`.
- [x] **Error Handling**
    - [x] Create `AppError` enum for structured error handling.
    - [x] Refactor all routes to use `AppError`.
- [x] **Optimization**
    - [x] Embed `user_id` in JWT to reduce database lookups.
    - [x] Centralize configuration management.

## Phase 5: File Upload & S3 Integration
**Goal**: Implement secure file uploads to S3 using API Key authentication.

- [ ] **S3 Integration**
    - [ ] Setup AWS SDK (`aws-sdk-s3`).
    - [ ] Create a helper service for S3 operations (upload, delete, presign).
- [ ] **File Models**
    - [ ] Create migration for `files` table (id, project_id, s3_key, filename, mime_type, size, status, variants_json).
- [ ] **Upload API (API Key Auth)**
    - [ ] Implement API Key middleware to resolve `project_id` from header.
    - [ ] `POST /upload/file`: Standard file upload.
        - [ ] Validates API Key.
        - [ ] Uploads file to S3 (`{projectId}/files/{fileId}.{ext}`).
        - [ ] Returns public URL immediately.
    - [ ] `POST /upload/image`: Image upload with processing.
        - [ ] Validates API Key.
        - [ ] Uploads original image to S3.
        - [ ] Calculates future variant paths (e.g., thumbnail, medium).
        - [ ] Returns JSON with original URL and *future* variant URLs.
        - [ ] Enqueues `ImageProcessingJob`.

## Phase 6: Asynchronous Image Processing
**Goal**: Handle image resizing and optimization in the background.

- [ ] **Queue System**
    - [ ] Choose a queue backend (Redis with `sidekiq-rs` or simple DB-backed queue).
    - [ ] Define job structure: `ImageProcessingJob { file_id, variants_config }`.
- [ ] **Worker Service**
    - [ ] Create a background worker that polls/listens for jobs.
    - [ ] Implement image processing using `image` crate (resize, format conversion).
    - [ ] Fetch original from S3, generate variants.
    - [ ] Upload generated variants to S3 (matching the paths returned in Phase 5).
    - [ ] Update file status in DB to "ready".
- [ ] **Integration**
    - [ ] Trigger a job upon successful image upload in Phase 5.

## Phase 7: File Retrieval & Serving
**Goal**: Serve files and specific image variants.

- [ ] **Retrieval API**
    - [ ] `GET /files/:id`: Get file metadata and public URL.
    - [ ] `GET /files/:id/content`: Redirect to S3 presigned URL or proxy content.
    - [ ] Support query params for variants (e.g., `?variant=thumbnail`).
    - [ ] Implement "Lazy Processing": If variant doesn't exist, trigger job and return original/placeholder.

## Phase 8: Cleanup & Advanced Features
**Goal**: Maintenance tasks and polish.

- [ ] **Deletion Logic**
    - [ ] Implement hard delete (remove from DB + S3).
    - [ ] Implement cascade delete (Project -> Files).
- [ ] **Cleanup Jobs**
    - [ ] Scheduled job to remove "soft deleted" items after X days.
    - [ ] Scheduled job to clean orphaned S3 objects.

## Phase 9: API Documentation
**Goal**: Provide interactive API documentation via Swagger/OpenAPI.

- [x] **OpenAPI Integration**
    - [x] Add `utoipa`, `utoipa-axum`, and `utoipa-swagger-ui` dependencies.
    - [x] Configure OpenAPI specification with API metadata.
    - [x] Define JWT Bearer security scheme.
- [x] **Endpoint Documentation**
    - [x] Document all authentication endpoints (`/auth/login`, `/auth/refresh`, `/auth/logout`, `/auth/me`).
    - [x] Document all user management endpoints (`/users`).
    - [x] Add request/response schema annotations.
- [x] **Swagger UI**
    - [x] Setup Swagger UI at `/swagger-ui`.
    - [x] Configure interactive API testing interface.
    - [x] Update README with Swagger UI access instructions.
