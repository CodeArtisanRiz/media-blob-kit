# MediaBlobKit Implementation Roadmap

This document outlines the step-by-step implementation plan for the MediaBlobKit project.

## Phase 1: Foundation & Infrastructure Setup [Completed]
**Goal**: Set up the core application structure, database connection, and configuration.

- [x] **Project Configuration**
    - [x] Setup `dotenv` for environment variables (DB_URL, JWT_SECRET, AWS_ACCESS_KEY, etc.).
    - [x] Create a `Config` struct to load and validate env vars.
- [x] **Database Setup**
    - [x] Choose an ORM/Query Builder (e.g., `sqlx` or `sea-orm`).
    - [x] Setup PostgreSQL database.
    - [x] Create initial migration for `users` table.
- [x] **Logging & Tracing**
    - [x] Implement structured logging with format: `Module | METHOD /path | context | res=code`
    - [x] Log all requests with user/project context
    - [x] Log all errors (4xx, 5xx) with full context

## Phase 2: Authentication & User Management [Completed]
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


## Phase 3: Project Management [Completed]
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


## Phase 4: Core Improvements & Refactoring [Completed]
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

## Phase 5: File Upload & S3 Integration [Completed]
**Goal**: Implement secure file uploads to S3 using API Key authentication.

- [x] **S3 Integration**
    - [x] Setup AWS SDK (`aws-sdk-s3`).
    - [x] Create a helper service for S3 operations (upload, delete, presign).
    - [x] **Fix**: Explicitly set credentials to resolve `SignatureDoesNotMatch`.
    - [x] **Fix**: Enforce Public Bucket Policy (`s3:GetObject`) on upload to resolve `AccessDenied`.
- [x] **File Models**
    - [x] Create migration for `files` table (id, project_id, s3_key, filename, mime_type, size, status, variants_json).
- [x] **Upload API (API Key Auth)**
    - [x] Implement API Key middleware to resolve `project_id` from header.
    - [x] `POST /upload/file`: Standard file upload.
        - [x] Validates API Key.
        - [x] Uploads file to S3 (`{project_name}-{project_id}/files/{uuid}.{ext}`).
        - [x] Returns public URL immediately.
    - [x] `POST /upload/image`: Image upload with processing.
        - [x] Validates API Key.
        - [x] Uploads original image to S3 (`{project_name}-{project_id}/images/original/{uuid}.{ext}`).
        - [x] Calculates future variant paths based on `ProjectSettings`.
        - [x] Returns JSON with original URL and *future* variant URLs.

> [!NOTE]
> **File Storage Structure**:
> All files are stored in a single S3 bucket defined by `S3_BUCKET_NAME`.
> - **Files**: `{project_name}-{project_id}/files/{uuid}.{ext}`
> - **Images**: `{project_name}-{project_id}/images/original/{uuid}.{ext}`
> - **Variants**: `{project_name}-{project_id}/images/{variant_name}/{uuid}.{ext}`
>
> **UUID Naming**: Original filenames are ignored for storage to prevent collisions. A UUID is generated for every upload. The original filename is preserved in the database.

## Phase 6: Jobs API [Completed]
**Goal**: Monitor and manage background processing jobs.

- [x] **Jobs Table**
    - [x] Create migration for `jobs` table (id, file_id, status, payload, created_at, updated_at).
- [x] **Jobs API (Project Level)**
    - [x] `GET /jobs`: List jobs for the authenticated project.
    - [x] Support filtering by status (`pending`, `processing`, `completed`, `failed`).
    - [x] Support pagination.
- [x] **Admin Jobs API (System Level)**
    - [x] `GET /admin/jobs`: List jobs grouped by project.
    - [x] Role-based access:
        - Su: View all projects.
        - Admin: View owned projects.
    - [x] Support pagination per project.

## Phase 7: Asynchronous Image Processing [Completed]
**Goal**: Handle image resizing and optimization in the background.

- [x] **Queue System**: DB-backed queue using `jobs` table.
- [x] **Worker Service**: Background worker polling for jobs.
- [x] **Image Processing**: Resizing and format conversion (AVIF, WebP).
- [x] Support for `fit` modes (contain, cover, stretch).
- [x] **Integration**: Automatic job creation on upload.
- [x] **Worker Recovery**: Automatic reset of stale 'processing' jobs on startup.

## Phase 8: Parallel Job Processing [Completed]
**Goal**: Increase throughput by processing multiple jobs concurrently within a single worker instance.

- [x] **Configuration**
    - [x] Add `WORKER_CONCURRENCY` env var (default to 1).
    - [x] Update `Config` struct.
- [x] **Concurrency Logic**
    - [x] Use `tokio::sync::Semaphore` to limit concurrent jobs.
    - [x] Spawn distinct tasks for each job `handle_job` call.
    - [x] Ensure database connections are efficiently managed using the pool.

## Phase 9: File Retrieval & Serving [Completed]
**Goal**: Serve files and specific image variants.

- [x] **Retrieval API**
    - [x] `GET /files`: List all files (Paginated, query param `?project_id=<id>`).
    - [x] `GET /files/:id`: Get file metadata and public URL.
    - [x] `GET /files/:id/content`: Redirect to S3 presigned URL or proxy content.
        - [x] Support query params for variants (e.g., `?variant=thumbnail`).

## Phase 10: Cleanup & Advanced Features [Completed]
**Goal**: Maintenance tasks and polish.

- [x] **Deletion Logic**
    - [x] `DELETE /files/:id`: Hard delete file.
        - [x] Verify ownership.
        - [x] Delete from S3 (original + all variants).
        - [x] Delete from DB.
    - [x] `DELETE /projects/:id`: Support `?permanent=true` for hard delete.
        - [x] If permanent:
            - [x] Iterate all project files.
            - [x] Delete each file from S3.
            - [x] Delete project (DB cascade handles file rows, or manual delete).
- [x] **Cleanup Jobs**
    - [x] `src/services/cleanup.rs`: Background cleanup service.
    - [x] Job: Remove "soft deleted" projects > 30 days.
    - [ ] Job: Scan S3 vs DB to find orphaned objects (advanced).
- [x] **Variant Synchronization**
    - [x] `POST /projects/:id/sync-variants`: Manual trigger.
    - [x] Worker: `SyncProjectVariants` -> Spawn `SyncFileVariants`.
    - [x] Worker: `SyncFileVariants` -> Regenerate/Upload.
## Phase 11: API Documentation
**Goal**: Provide interactive API documentation via Swagger/OpenAPI.

- [x] **OpenAPI Integration**
    - [x] Add `utoipa`, `utoipa-axum`, and `utoipa-swagger-ui` dependencies.
    - [x] Configure OpenAPI specification with API metadata.
    - [x] Define JWT Bearer security scheme.
- [x] **Endpoint Documentation**
    - [x] Document all authentication endpoints (`/auth/login`, `/auth/refresh`, `/auth/logout`, `/auth/me`).
    - [x] Document all user management endpoints (`/users`).
    - [x] Document all file retrieval endpoints (`/files`).
    - [x] Add request/response schema annotations.
- [x] **Swagger UI**
    - [x] Setup Swagger UI at `/swagger-ui`.
    - [x] Configure interactive API testing interface.
    - [x] Update README with Swagger UI access instructions.
