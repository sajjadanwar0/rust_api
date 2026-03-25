pub mod handlers;
pub mod models;

use std::env;
use std::net::SocketAddr;
use axum::http::Method;
use axum::Router;
use axum::routing::get;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    db_pool: PgPool,
}



#[tokio::main]
async fn main() {
    // 1. Setup Environment
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // 2. Create Connection Pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database connection pool");
    println!("🚀 Database connection pool initialized.");

    // 3. Create Table (Migration)
    // Running this on startup ensures the DB structure exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id SERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT FALSE
        );
        "#
    )
        .execute(&pool)
        .await
        .expect("Failed to create 'todos' table");
    println!("🚀 'todos' table is ready.");

    // 4. Initialize State
    let app_state = AppState { db_pool: pool };

    // 5. Build Router
    use handlers::{
        create_todo_db, delete_todo_db, get_all_todos_db, get_todo_db, update_todo_db,
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([axum::http::header::CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/todos",
               get(get_all_todos_db)
                   .post(create_todo_db)
        )
        // Use :id for Axum dynamic path syntax
        .route("/todos/{id}",
               get(get_todo_db)
                   .patch(update_todo_db)
                   .delete(delete_todo_db)
        )
        .with_state(app_state)
        .layer(cors);

    // 6. Bind and Serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("🚀 Server listening on https://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


