use axum::Router;
use std::sync::Arc;

#[allow(warnings, unused)]
mod prisma;

mod auth;
pub(crate) use auth::Session;

mod user;

macro_rules! option_vec {
    [$($x: expr),+ $(,)?] => {
        {
            let mut items = Vec::new();
            $(
                if let Some(x) = $x { items.push(x) }
            )*
            items
        }
    };
}
pub(crate) use option_vec;

#[derive(Clone)]
pub struct AppState {
    client: Arc<prisma::PrismaClient>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .nest("/", auth::router())
        .nest("/", user::router())
        .with_state(AppState {
            client: Arc::new(prisma::new_client().await.unwrap()),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
