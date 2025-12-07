# Media Blob Kit

A high-performance media blob management system built with Rust, Axum, SeaORM, and PostgreSQL. Features secure authentication, role-based access control, and efficient project management.

## Prerequisites

- Rust (latest stable)
- PostgreSQL
- AWS S3/S3 Compatible Storage

### Hardware Requirements

| Spec | Minimum | Recommended |
|------|---------|-------------|
| **CPU** | 1 vCPU | 2 vCPU (for parallel calls & background worker) |
| **RAM** | 512 MB | 2 GB (safe buffer for heavy image processing) |
| **Disk** | Minimal | 10 GB (for OS, logs, and temp buffering) |

**Scaling & Concurrency:**
- **Per Instance (Configurable)**: By default, a single instance processes one job at a time. This can be increased via the `WORKER_CONCURRENCY` environment variable (e.g., `WORKER_CONCURRENCY=4`) to process multiple jobs in parallel.
- **Horizontal Scaling**: To process multiple jobs in parallel across servers, simply run multiple instances of the application. The `SKIP LOCKED` database queue ensures they distribute the load automatically.

> **Note on Safety**: You can run as many worker instances as you like. We use PostgreSQL's `FOR UPDATE SKIP LOCKED` clause, which guarantees that **no two workers will ever pick up the same job**, even if they query the database at the exact same millisecond.

## Setup

1.  Create a `.env` file with your DB credentials:
    ```env
    DATABASE_URL=postgres://username:yoursecretpassword@localhost:port/database_name
    JWT_SECRET=your_super_secret_jwt_key
    AWS_REGION=us-east-1                    # Required (use 'us-east-1' for MinIO if unsure)
    AWS_ACCESS_KEY_ID=your_access_key
    AWS_SECRET_ACCESS_KEY=your_secret_key
    S3_BUCKET_NAME=your_bucket_name
    S3_ENDPOINT=https://minio.example.com   # Optional (Required for MinIO)
    WORKER_CONCURRENCY=4
    ```

2.  Run migrations:
    ```bash
    cargo run -- migrate
    ```

## Usage

### Create Superuser

To create a new superuser:

```bash
cargo run -- create-superuser --username <your_username>
```

You will be prompted to enter a password.

### Run Server

To start the server:

```bash
# Development (Fast compilation, slow execution)
cargo run

# Production / Deployment (Optimized speed for image processing)
cargo run --release
```

The server will start on `http://0.0.0.0:3000`.


## 📦 Project Overview
MediaBlobKit is a Rust-based web application built with Axum, SeaORM, and PostgreSQL. It's designed to be a media blob management system with user authentication and role-based access control.

## 🏗️ Project Structure
```text
media-blob-kit/
├── src/
│   ├── main.rs                 # App entry point with CLI commands
│   ├── config.rs               # Configuration loading
│   ├── error.rs                # Application error handling
│   ├── pagination.rs           # Pagination utilities
│   ├── entities/               # Database entity models
│   │   ├── mod.rs
│   │   ├── user.rs             # User entity
│   │   ├── refresh_token.rs    # Refresh token entity
│   │   ├── project.rs          # Project entity
│   │   ├── api_key.rs          # API Key entity
│   │   ├── file.rs             # File entity
│   │   └── job.rs              # Job entity
│   ├── middleware/             # Auth & authorization middleware
│   │   ├── mod.rs
│   │   ├── auth.rs             # JWT authentication middleware
│   │   ├── api_key.rs          # API Key validation
│   │   └── role.rs             # Role-based authorization
│   ├── models/                 # Shared data models
│   │   ├── mod.rs
│   │   └── settings.rs         # Project settings models
│   ├── routes/                 # API route handlers
│   │   ├── mod.rs              # Route registration
│   │   ├── auth.rs             # Auth endpoints
│   │   ├── users.rs            # User management
│   │   ├── projects.rs         # Project management
│   │   ├── api_keys.rs         # API Key management
│   │   ├── upload.rs           # File & Image upload handlers
│   │   ├── jobs.rs             # Jobs API
│   │   └── home.rs             # Root HTML page
│   ├── services/               # core logic services
│   │   ├── mod.rs
│   │   ├── s3.rs               # AWS S3 integration
│   │   └── worker.rs           # Background worker service
│   └── utils/                  # Helper utilities
│       ├── mod.rs
│       └── image_processor.rs  # Image processing logic
├── migration/                  # Database migrations
│   ├── src/
│   │   ├── lib.rs
│   │   └── m*                  # Migration files
│   └── Cargo.toml
├── utils/
│   └── reset_db.rs             # Database reset utility
├── .env                        # Environment variables
├── Cargo.toml                  # Project dependencies
├── IMPLEMENTATION.md           # Comprehensive roadmap
└── README.md                   # User documentation
```
## 🔧 Current Features (Implemented)

### Database Management
- PostgreSQL integration via SeaORM
- Users table: id, username, password, role, created_at
- Refresh tokens table: token_hash, user_id, expires_at, revoked
- Migration system with reset capability
- Foreign key relationships with cascade delete

### Authentication System
- Argon2 password hashing
- JWT access tokens (15-minute expiration)
- Refresh tokens (1-day expiration) with SHA-256 hashing
- Stateless authentication with middleware
- Descriptive error messages

### Authorization & Middleware
- JWT authentication middleware
- Role-based authorization (superuser-only routes)
- Proper route grouping with scoped middleware

### User Management (Su-only)
- Create admin/user accounts
- List all users (Paginated)
- Delete users (prevents self-deletion)

### File Uploads & Storage
- **S3 Integration**: Seamless upload to AWS S3 or MinIO.
- **Project Isolation**: Files are organized by project folders within a single bucket.
- **Public Access**: Automatic public bucket policy configuration.
- **Image Processing**:
    - Automatic variant path calculation.
    - Asynchronous resizing and format conversion (AVIF, WebP, JPEG, PNG).
    - Background worker service for offloading heavy tasks.
- **Security**:
    - API Key authentication for uploads.
    - UUID-based filenames to prevent collisions and malicious naming.

### Core Improvements
- **Pagination**: Standardized pagination with metadata (total items, pages) for all list endpoints.
- **Configuration**: Centralized, type-safe configuration loading from environment variables.
- **Error Handling**: Unified, structured error responses across the entire API.
- **Performance**: Optimized authentication context to reduce database lookups.

### Structured Logging
All API requests and errors are logged with a consistent, structured format:

```
Module | METHOD /path | context | res=code
```

**Example Logs:**
```
Auth | POST /auth/login | user=riz | res=200
Auth | GET /auth/me | user=riz | res=200
User | POST /users | user=riz | created=john | res=201
User | GET /users | user=riz | count=5 | res=200
User | DELETE /users/uuid | user=riz | res=404 | User not found
Project | POST /projects | user=riz | name=myapp | res=201
Project | GET /projects | user=riz | count=3 | res=200
ApiKey | POST /projects/uuid/keys | user=riz | res=201
Upload | POST /upload/file | project=myapp | file=doc.pdf | res=200
Upload | POST /upload/image | project=myapp | file=uuid | res=200
Jobs | GET /jobs | project=myapp | count=5 | res=200
Jobs | GET /admin/jobs | user=riz | projects=3 | res=200
Error | res=401 | Missing API Key
Error | res=404 | User not found
```

### API Documentation

**Interactive Swagger UI** is available at: **`http://localhost:3000/swagger-ui`**

The Swagger UI provides:
- Complete API endpoint documentation with request/response schemas
- Interactive testing interface - try out API calls directly from your browser
- JWT Bearer token authentication support
- Organized endpoints by tags (General, Authentication, User Management)

To test authenticated endpoints:
1. First login via `POST /auth/login` to obtain an access token
2. Click the "Authorize" button at the top of the Swagger UI
3. Enter your access token in the format: `Bearer <your_access_token>`
4. Now you can test protected endpoints like `/auth/me` and `/users`


### Role-Based System
- su (superuser): CLI-only creation, full user management access
- admin: Project and file management (Phase 3)
- user: Basic access (TBD)

### Project Management
- Create and manage projects
- Generate and manage API keys
- Paginated list views for projects and keys

### CLI Commands
- `cargo run -- migrate` - Apply migrations
- `cargo run -- reset` - Refresh database
- `cargo run -- create-superuser --username <name>` - Create superuser account
- `cargo run` - Start the web server
- `cargo check` - Check for errors

## 📚 Tech Stack
Framework: Axum 0.8.7
Database ORM: SeaORM 1.1.2
Database: PostgreSQL
Password Hashing: Argon2 0.5.3
JWT: jsonwebtoken 9.3.0
Async Runtime: Tokio 1.48.0
CLI: Clap 4.5.21
AWS SDK: aws-sdk-s3 1.60.0
Image Processing: image 0.25.9

## 🚀 Implementation Roadmap

The [`IMPLEMENTATION.md`](IMPLEMENTATION.md) file outlines a comprehensive 10-phase plan for MediaBlobKit.

**Phase 1: Foundation & Infrastructure**
- ✅ Completed: Database setup (PostgreSQL + SeaORM)
- ✅ Completed: Environment configuration with `.env`
- ✅ Completed: Migration system with reset capability
- ✅ Completed: Config struct for env validation

**Phase 2: Authentication & User Management**
- ✅ Completed: Dual-token authentication (access + refresh tokens)
- ✅ Completed: JWT middleware with role-based authorization
- ✅ Completed: User management API (su-only)
- ✅ Completed: Password hashing with Argon2
- ✅ Completed: Complete API documentation

**Phase 3: Project Management**
- ✅ Completed: Project models & migrations
- ✅ Completed: Project creation and settings API
- ✅ Completed: Project-based file organization
- ✅ Completed: Admin/user project permissions
- ✅ Completed: API Key management (Create, List, Revoke, Patch)

**Phase 4: Core Improvements**
- ✅ Completed: Pagination, Error Handling, Optimization

**Phase 5: File Upload & S3 Integration**
- ✅ Completed: AWS S3 integration setup
- ✅ Completed: File models & migrations
- ✅ Completed: Multipart upload endpoints (File & Image)
- ✅ Completed: Public bucket policy automation
- ✅ Completed: Project-specific S3 folder structure

**Phase 6: Jobs API**
- ✅ Completed: Jobs table & migration
- ✅ Completed: Project-level Jobs API (`GET /jobs`)
- ✅ Completed: Admin-level Jobs Dashboard (`GET /admin/jobs`)
- ✅ Completed: Status filtering & Pagination

**Phase 7: Asynchronous Image Processing**
- ✅ Completed: Queue system (DB-backed)
- ✅ Completed: Background worker service
- ✅ Completed: Image resizing and optimization
- ✅ Completed: Variant generation and storage

**Phase 8: Parallel Job Processing**
- ✅ Completed: Configurable concurrency (env driven)
- ✅ Completed: Semaphore-based parallel execution

**Phase 9: File Retrieval & Serving**
- ✅ Completed: File metadata and URL endpoints
- ✅ Completed: S3 presigned URLs or proxy
- ✅ Completed: Image variant serving

**Phase 10: Cleanup & Advanced Features**
- ✅ Completed: Hard delete logic (Files)
- ✅ Completed: Cascade delete logic (Projects)
- ✅ Completed: Scheduled cleanup jobs
- ⏳ Pending: Orphaned S3 object cleanup (Advanced/Optional)
- ✅ Completed: API key generation for programmatic uploads

**Phase 11: API Documentation**
- ✅ Completed: OpenAPI Integration
- ✅ Completed: Endpoint Documentation
- ✅ Completed: Swagger UI

See [`IMPLEMENTATION.md`](IMPLEMENTATION.md) for detailed task breakdowns.

## 🔐 Security Notes
Password Hashing: Uses Argon2 (secure default)
Refresh Tokens: Hashed with SHA-256 before database storage
Access Token Expiration: 15 minutes
Refresh Token Expiration: 1 day
Database credentials stored in 
.env

**Note on Logout Behavior:** Due to the stateless nature of JWTs, access tokens remain valid until expiration even after logout. Logout immediately revokes the refresh token (preventing new access tokens), but existing access tokens will continue to work for up to 15 minutes. For immediate token revocation, consider implementing a Redis-based token blacklist.

### API Endpoints

#### Authentication

-   **`POST /auth/login`** - Login to get access and refresh tokens
    -   **Request Body:**
        ```json
        {
          "username": "your_username",
          "password": "your_password"
        }
        ```
    -   **Response:**
        ```json
        {
          "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
          "refresh_token": "dGhpcyBpcyBhIHJhbmRvbSB0b2tlbg==",
          "expires_in": 900
        }
        ```
    -   **Note:** Access token expires in 15 minutes (900 seconds)

-   **`POST /auth/refresh`** - Get a new access token using refresh token
    -   **Request Body:**
        ```json
        {
          "refresh_token": "dGhpcyBpcyBhIHJhbmRvbSB0b2tlbg=="
        }
        ```
    -   **Response:**
        ```json
        {
          "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
        }
        ```

-   **`POST /auth/logout`** - Revoke a refresh token
    -   **Request Body:**
        ```json
        {
          "refresh_token": "dGhpcyBpcyBhIHJhbmRvbSB0b2tlbg=="
        }
        ```
    -   **Response:**
        ```json
        {
          "message": "Logged out successfully"
        }
        ```

-   **`GET /auth/me`** - Get current user profile (requires authentication)
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Response:**
        ```json
        {
          "id": 1,
          "username": "riz",
          "role": "su",
          "created_at": "2024-12-01T10:00:00"
        }
        ```

#### User Management (Su-only)

All user management endpoints require superuser authentication.

-   **`POST /users`** - Create a new user
    -   **Headers:** `Authorization: Bearer <access_token>` (su role required)
    -   **Request Body:**
        ```json
        {
          "username": "john_admin",
          "password": "secure123",
          "role": "admin"
        }
        ```
    -   **Valid Roles:** `"admin"` or `"user"` (cannot create `"su"` via API)
    -   **Response (201 Created):**
        ```json
        {
          "id": 2,
          "username": "john_admin",
          "role": "admin",
          "created_at": "2024-12-01T12:00:00"
        }
        ```

-   **`GET /users`** - List all users
    -   **Headers:** `Authorization: Bearer <access_token>` (su role required)
    -   **Query Params:** `?page=1&limit=10`
    -   **Response:**
        ```json
        {
          "data": [
            {
              "id": 1,
              "username": "riz",
              "role": "su",
              "created_at": "2024-12-01T10:00:00"
            },
            {
              "id": 2,
              "username": "john_admin",
              "role": "admin",
              "created_at": "2024-12-01T12:00:00"
            }
          ],
          "total_items": 2,
          "total_pages": 1,
          "current_page": 1,
          "page_size": 10
        }
        ```

-   **`DELETE /users/{id}`** - Delete a user
    -   **Headers:** `Authorization: Bearer <access_token>` (su role required)
    -   **Response:**
        ```json
        {
          "message": "User deleted successfully"
        }
        ```
    -   **Note:** Cannot delete yourself

#### Project Management

-   **`GET /projects`** - List projects (Paginated)
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Query Params:** `?page=1&limit=10`
    -   **Response:**
        ```json
        {
          "data": [
            {
              "id": "uuid...",
              "name": "My Project",
              "owner_id": "uuid...",
              "settings": {},
              "created_at": "..."
            }
          ],
          "total_items": 1,
          "total_pages": 1,
          "current_page": 1,
          "page_size": 10
        }
        ```

-   **`POST /projects`** - Create a new project
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Request Body:**
        ```json
        {
          "name": "New Project",
          "settings": { "quota": 100 }
        }
        ```

-   **`GET /projects/{id}`** - Get project details
    -   **Headers:** `Authorization: Bearer <access_token>`

-   **`PUT /projects/{id}`** - Update project
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Note:** Currently, updating `image_variants` in settings does *not* automatically reprocess existing files. In the future, this will trigger a background job to sync variants.

-   **`DELETE /projects/{id}`** - Delete project (Soft delete)
    -   **Headers:** `Authorization: Bearer <access_token>`

#### Image Variant Configuration

You can configure image variants in the `Project` settings. The worker will automatically process uploaded images based on these rules.

**Supported Fit Modes:**
- `contain` (Default): Resizes to fit within constraints, preserving aspect ratio. No cropping.
- `cover` / `center-crop`: Resizes to fill constraints, cropping the excess from the center.
- `fill` / `stretch` / `exact`: Forces exact dimensions, ignoring aspect ratio.

**Example Configuration:**

```json
{
  "variants": {
    "thumb": {
      "format": "webp",
      "width": 150,
      "height": 150,
      "fit": "cover",
      "quality": 80
    },
    "medium": {
      "format": "webp",
      "width": 800
      // height auto-calculated
    },
    "hero-avif": {
      "format": "avif",
      "width": 1600,
      "quality": 70
    }
  }
}
```

#### API Keys

-   **`GET /projects/{id}/keys`** - List API keys (Paginated)
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Query Params:** `?page=1&limit=10`

-   **`POST /projects/{id}/keys`** - Create API key
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Request Body:**
        ```json
        {
          "name": "Production Key",
          "expires_at": "2025-01-01T00:00:00Z"
        }
        ```
    -   **Response:** Returns the raw API key (only once!)

-   **`PATCH /projects/{id}/keys/{key_id}`** - Enable/Disable API key
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Request Body:**
        ```json
        {
          "is_active": false
        }
        ```
    -   **Response:**
        ```json
        {
          "message": "API Key updated successfully"
        }
        ```

-   **`DELETE /projects/{id}/keys/{key_id}`** - Permanently delete API key
    -   **Headers:** `Authorization: Bearer <access_token>`

#### File Uploads

-   **`POST /upload/file`** - Standard File Upload
    -   **Headers:** `x-api-key: <your_project_api_key>`
    -   **Body:** `multipart/form-data` with field `file`
    -   **Response:**
        ```json
        {
          "id": "uuid...",
          "url": "https://s3.../project-id/files/uuid.pdf",
          "filename": "original.pdf",
          "mime_type": "application/pdf",
          "size": 1024
        }
        ```

-   **`POST /upload/image`** - Image Upload
    -   **Headers:** `x-api-key: <your_project_api_key>`
    -   **Body:** `multipart/form-data` with field `file`
    -   **Response:**
        ```json
        {
          "id": "uuid...",
          "original_url": "https://s3.../project-id/images/original/uuid.jpg",
          "variants": {
            "thumbnail": "https://s3.../project-id/images/thumbnail/uuid.jpg",
            "medium": "https://s3.../project-id/images/medium/uuid.webp"
          }
        }
        ```

#### Jobs API

-   **`GET /jobs`** - List jobs for the authenticated project
    -   **Headers:** `x-api-key: <your_project_api_key>`
    -   **Query Params:** `?status=pending&page=1&limit=10`
    -   **Response:**
        ```json
        {
          "My Project": {
            "project_id": "uuid...",
            "jobs": [
              {
                "id": "uuid...",
                "status": "pending",
                "payload": { ... },
                "created_at": "..."
              }
            ],
            "total_items": 1,
            "total_pages": 1,
            "current_page": 1,
            "page_size": 10
          }
        }
        ```

-   **`GET /admin/jobs`** - Admin Jobs Dashboard
    -   **Headers:** `Authorization: Bearer <access_token>`
    -   **Query Params:** `?page=1&limit=10`
    -   **Response:** Returns a map of projects with their paginated jobs.
        ```json
        {
          "Project A": {
            "project_id": "uuid...",
            "jobs": [ ... ],
            "total_items": 5,
            ...
          },
          "Project B": { ... }
        }
        ```

#### General

-   **`GET /`** - Health check
    -   Returns HTML welcome page
