# Media Blob Kit

A high-performance media blob management system built with Rust, Axum, SeaORM, and PostgreSQL. Features secure authentication, role-based access control, and efficient project management.

## Prerequisites

- Rust (latest stable)
- PostgreSQL
- AWS S3/S3 Compatible Storage

## Setup

1.  Create a `.env` file with your DB credentials:
    ```env
    DATABASE_URL=postgres://username:yoursecretpassword@localhost:port/database_name
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
cargo run
```

The server will start on `http://0.0.0.0:3000`.


## 📦 Project Overview
MediaBlobKit is a Rust-based web application built with Axum, SeaORM, and PostgreSQL. It's designed to be a media blob management system with user authentication and role-based access control.

## 🏗️ Project Structure
```
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
│   │   └── api_key.rs          # API Key entity
│   ├── middleware/             # Auth & authorization middleware
│   │   ├── mod.rs
│   │   ├── auth.rs             # JWT authentication middleware
│   │   └── role.rs             # Role-based authorization
│   └── routes/                 # API route handlers
│       ├── mod.rs              # Route registration
│       ├── auth.rs             # Auth endpoints
│       ├── users.rs            # User management
│       ├── projects.rs         # Project management
│       ├── api_keys.rs         # API Key management
│       └── home.rs             # Root HTML page
├── migration/                  # Database migrations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── m20240101_000001_create_user_table.rs
│   │   ├── m20241201_000002_create_refresh_tokens_table.rs
│   │   ├── m20241202_000003_create_projects_table.rs
│   │   └── m20241202_000004_create_api_keys_table.rs
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

### Core Improvements
- **Pagination**: Standardized pagination with metadata (total items, pages) for all list endpoints.
- **Configuration**: Centralized, type-safe configuration loading from environment variables.
- **Error Handling**: Unified, structured error responses across the entire API.
- **Performance**: Optimized authentication context to reduce database lookups.

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
## 🚀 Implementation Roadmap

The [`IMPLEMENTATION.md`](IMPLEMENTATION.md) file outlines a comprehensive 7-phase plan for MediaBlobKit.

**Phase 1: Foundation & Infrastructure**
- ✅ Completed: Database setup (PostgreSQL + SeaORM)
- ✅ Completed: Environment configuration with `.env`
- ✅ Completed: Migration system with reset capability
- ⏳ Pending: Config struct for env validation
- ⏳ Pending: Structured logging with tracing-subscriber

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

**Phase 4: File Upload & S3 Integration**
- ✅ Completed: AWS S3 integration setup
- ✅ Completed: File models & migrations
- ✅ Completed: File metadata management
- ✅ Completed: Multipart upload endpoints

**Phase 5: Asynchronous Image Processing**
- ⏳ Pending: Queue system (Redis or DB-backed)
- ⏳ Pending: Background worker service
- ⏳ Pending: Image resizing and optimization
- ⏳ Pending: Variant generation and storage

**Phase 6: File Retrieval & Serving**
- ⏳ Pending: File metadata and URL endpoints
- ⏳ Pending: S3 presigned URLs or proxy
- ⏳ Pending: Image variant serving
- ⏳ Pending: Lazy processing for on-demand variants

**Phase 7: Cleanup & Advanced Features**
- ⏳ Pending: Hard and cascade delete logic
- ⏳ Pending: Scheduled cleanup jobs
- ⏳ Pending: Orphaned S3 object cleanup
- ⏳ Pending: API key generation for programmatic uploads

**Phase 8: API Documentation**
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

#### General

-   **`GET /`** - Health check
    -   Returns HTML welcome page
