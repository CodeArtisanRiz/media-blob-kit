# Media Blob Kit

A Rust/Axum application with SeaORM and PostgreSQL.

## Prerequisites

- Rust (latest stable)
- PostgreSQL running on port 5432

## Setup

1.  Create a `.env` file with your DB credentials:
    ```env
    DATABASE_URL=postgres://username:yoursecretpassword@localhost:5432/media_blob_kit
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
media-blob-kit/
├── src/
│   ├── main.rs                 # App entry point with CLI commands
│   ├── entities/               # Database entity models
│   │   ├── mod.rs
│   │   └── user.rs            # User entity with roles (su, admin, user)
│   └── routes/                # API route handlers
│       ├── mod.rs             # Route registration
│       ├── auth.rs            # Login endpoint with JWT
│       └── home.rs            # Root HTML page
├── migration/                  # Database migrations
│   ├── src/
│   │   ├── lib.rs
│   │   └── m20240101_000001_create_user_table.rs
│   └── Cargo.toml
├── utils/
│   └── reset_db.rs            # Database reset utility
├── .env                        # Database connection string
├── Cargo.toml                  # Project dependencies
├── IMPLEMENTATION.md           # Comprehensive roadmap
└── README.md                   # User documentation
🔧 Current Features (Implemented)
Database Management
PostgreSQL integration via SeaORM
Users table with id, username, password, role, created_at
Migration system with reset capability
Authentication System
Argon2 password hashing
JWT token generation (1-hour expiration)
Role-based system: su (superuser), admin, user
CLI Commands
cargo run -- migrate - Apply migrations
cargo run -- reset - Refresh database
cargo run -- create-superuser --username <name> - Create superuser
cargo run - Start the web server
API Endpoints
GET / - Health check with HTML page
POST /login - Authentication endpoint (returns JWT)
## 📚 Tech Stack
Framework: Axum 0.8.7
Database ORM: SeaORM 1.1.2
Database: PostgreSQL
Password Hashing: Argon2 0.5.3
JWT: jsonwebtoken 9.3.0
Async Runtime: Tokio 1.48.0
CLI: Clap 4.5.21
## 🚀 Implementation Roadmap
The 
IMPLEMENTATION.md
 file outlines a comprehensive 7-phase plan:

Completed:

✅ Phase 1: Foundation (partial - DB setup done)
✅ Phase 2: Auth (partial - login & password hashing done)
Planned:

Phase 2 (remaining): Registration, profile endpoint, auth middleware
Phase 3: Project management system
Phase 4: S3 file uploads
Phase 5: Async image processing with queues
Phase 6: File retrieval and serving
Phase 7: Cleanup jobs and API keys
## 🔐 Security Notes
Password Hashing: Uses Argon2 (secure default)
Refresh Tokens: Hashed with SHA-256 before database storage
Access Token Expiration: 1 hour
Refresh Token Expiration: 7 days
Database credentials stored in 
.env

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
          "expires_in": 3600
        }
        ```
    -   **Note:** Access token expires in 1 hour (3600 seconds)

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

#### General

-   **`GET /`** - Health check
    -   Returns HTML welcome page
