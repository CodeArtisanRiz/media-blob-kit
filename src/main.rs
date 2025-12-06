mod entities;
mod routes;
mod middleware;
pub mod config;
mod error;
mod pagination;
pub mod services;
pub mod models;
pub mod utils;



use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use clap::{Parser, Subcommand};
use entities::user;
use migration::{Migrator, MigratorTrait};
use routes::create_routes;
use sea_orm::{ActiveModelTrait, Database, Set};
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply pending migrations
    Migrate,
    /// Reset database (refresh migrations)
    Reset,
    /// Create a superuser
    CreateSuperuser {
        #[arg(short, long)]
        username: String,
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // Initialize config
    let config = config::get_config();
    
    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Migrate) => {
            Migrator::up(&db, None).await.expect("Migration failed");
            println!("Migrations applied successfully");
        }
        Some(Commands::Reset) => {
            Migrator::refresh(&db).await.expect("Migration refresh failed");
            println!("Database reset successfully");
        }
        Some(Commands::CreateSuperuser { username }) => {
            let password = rpassword::prompt_password("Enter password: ").unwrap();
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            let user = user::ActiveModel {
                id: Set(Uuid::new_v4()),
                username: Set(username.clone()),
                password: Set(password_hash),
                role: Set(user::Role::Su),
                created_at: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            };

            match user.insert(&db).await {
                Ok(_) => println!("Superuser '{}' created successfully", username),
                Err(e) => eprintln!("Failed to create superuser: {}", e),
            }
        }
        None => {
            // build our application using the routes module
            let app = create_routes(db.clone());

            // Spawn background worker
            let worker_db = db.clone();
            tokio::spawn(async move {
                let worker = services::worker::Worker::new(worker_db).await;
                worker.run().await;
            });

            // run our app with hyper, listening globally on port 3000
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            println!("Listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        }
    }
}
