# MediaBlobKit Implementation Roadmap

This document outlines the step-by-step implementation plan for the MediaBlobKit project.

## Phase 1: Foundation & Infrastructure Setup
**Goal**: Set up the core application structure, database connection, and configuration.

- [x] **Project Configuration**
    - [x] Setup `dotenv` for environment variables (DB_URL, JWT_SECRET, AWS_ACCESS_KEY, etc.).
    - [ ] Create a `Config` struct to load and validate env vars.
- [x] **Database Setup**
    - [x] Choose an ORM/Query Builder (e.g., `sqlx` or `sea-orm`).
    - [x] Setup PostgreSQL database.
    - [x] Create initial migration for `users` table.
- [ ] **Logging & Tracing**
    - [ ] Configure `tracing-subscriber` for structured logging.

## Phase 2: Authentication & User Management
**Goal**: Implement secure user registration, login, and profile management.

- [x] **User Models & Migrations**
    - [x] Define `User` struct.
    - [x] Create migration for `users` (id, email, password_hash, created_at, etc.).
- [x] **Auth Logic**
    - [x] Implement password hashing using `argon2` or `bcrypt`.
    - [x] Implement JWT token generation and validation.
- [ ] **Auth Endpoints**
    - [ ] `POST /auth/register`: Create new user.
    - [x] `POST /auth/login`: Validate credentials and return JWT.
    - [ ] `GET /auth/me`: Get current user profile (protected).
- [ ] **Middleware**
    - [ ] Create an `AuthMiddleware` to validate JWT and inject `UserId` into extensions.

## Phase 3: Project Management
**Goal**: Allow users to create and manage projects with specific settings.

- [ ] **Project Models & Migrations**
    - [ ] Create migration for `projects` table (id, owner_id, name, settings_json, etc.).
- [ ] **Project API**
    - [ ] `POST /projects`: Create a new project.
    - [ ] `GET /projects`: List user's projects.
    - [ ] `GET /projects/:id`: Get project details.
    - [ ] `PUT /projects/:id`: Update project settings (image variants, quotas).
    - [ ] `DELETE /projects/:id`: Soft delete project.

## Phase 4: File Upload & S3 Integration
**Goal**: Implement generic file uploads to S3.

- [ ] **S3 Integration**
    - [ ] Setup AWS SDK (`aws-sdk-s3`).
    - [ ] Create a helper service for S3 operations (upload, delete, presign).
- [ ] **File Models**
    - [ ] Create migration for `files` table (id, project_id, s3_key, filename, mime_type, size, status).
- [ ] **Upload API**
    - [ ] `POST /projects/:id/upload`: Multipart upload handler.
    - [ ] Stream file to S3 bucket under `{projectId}/files/{fileId}.{ext}`.
    - [ ] Record metadata in DB.

## Phase 5: Asynchronous Image Processing
**Goal**: Handle image resizing and optimization in the background.

- [ ] **Queue System**
    - [ ] Choose a queue backend (Redis with `sidekiq-rs` or simple DB-backed queue).
    - [ ] Define job structure: `ImageProcessingJob { file_id, variants_config }`.
- [ ] **Worker Service**
    - [ ] Create a background worker that polls/listens for jobs.
    - [ ] Implement image processing using `image` crate (resize, format conversion).
    - [ ] Upload generated variants to S3 (`{projectId}/{variant}/{fileId}.{ext}`).
    - [ ] Update file status in DB to "ready".
- [ ] **Integration**
    - [ ] Trigger a job upon successful image upload in Phase 4.

## Phase 6: File Retrieval & Serving
**Goal**: Serve files and specific image variants.

- [ ] **Retrieval API**
    - [ ] `GET /files/:id`: Get file metadata and public URL.
    - [ ] `GET /files/:id/content`: Redirect to S3 presigned URL or proxy content.
    - [ ] Support query params for variants (e.g., `?variant=thumbnail`).
    - [ ] Implement "Lazy Processing": If variant doesn't exist, trigger job and return original/placeholder.

## Phase 7: Cleanup & Advanced Features
**Goal**: Maintenance tasks and polish.

- [ ] **Deletion Logic**
    - [ ] Implement hard delete (remove from DB + S3).
    - [ ] Implement cascade delete (Project -> Files).
- [ ] **Cleanup Jobs**
    - [ ] Scheduled job to remove "soft deleted" items after X days.
    - [ ] Scheduled job to clean orphaned S3 objects.
- [ ] **API Keys**
    - [ ] Allow generating API keys for projects to enable programmatic uploads.
