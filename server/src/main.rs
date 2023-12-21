use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use db::user;

mod db;

#[derive(Clone)]
struct AppState {
    client: Arc<db::PrismaClient>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/users", get(get_users))
        .with_state(AppState {
            client: Arc::new(db::new_client().await.unwrap()),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_users(State(AppState { client }): State<AppState>) -> Json<Vec<user::Data>> {
    Json(client.user().find_many(vec![]).exec().await.unwrap())
}
