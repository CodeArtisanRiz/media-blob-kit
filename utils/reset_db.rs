use sea_orm::{ConnectionTrait, Database, Statement, DbBackend};
use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    db.execute(Statement::from_string(
        DbBackend::Postgres,
        "DROP TABLE IF EXISTS \"user\" CASCADE;".to_owned(),
    ))
    .await
    .unwrap();
    db.execute(Statement::from_string(
        DbBackend::Postgres,
        "DROP TABLE IF EXISTS \"users\" CASCADE;".to_owned(),
    ))
    .await
    .unwrap();
    db.execute(Statement::from_string(
        DbBackend::Postgres,
        "DROP TABLE IF EXISTS seaql_migrations CASCADE;".to_owned(),
    ))
    .await
    .unwrap();
    println!("Database reset successfully");
}
