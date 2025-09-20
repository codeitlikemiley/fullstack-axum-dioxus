mod config;
mod errors;
use crate::errors::CustomError;
use axum::response::Html;
use axum::{extract::Extension, routing::get, Router};
use dioxus::dioxus_core::VirtualDom;
use pages::{
    render,
    users::{IndexPage, IndexPageProps},
};
use std::net::SocketAddr;
mod static_files;

#[tokio::main]
async fn main() {
    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // build our application with a route
    let app = Router::new()
        .route("/", get(users))
        .route("/static/*path", get(static_files::static_path))
        .layer(Extension(config))
        .layer(Extension(pool.clone()));

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on... {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn users(_pool: Extension<db::Pool>) -> Result<Html<String>, CustomError> {
    // Provide sample users when database queries are not available
    let sample_users = vec![
        db::User {
            id: "1".to_string(),
            email: "user1@example.com".to_string(),
        },
        db::User {
            id: "2".to_string(),
            email: "user2@example.com".to_string(),
        },
        db::User {
            id: "3".to_string(),
            email: "user3@example.com".to_string(),
        },
    ];

    let html = render(VirtualDom::new_with_props(
        IndexPage,
        IndexPageProps { users: sample_users },
    ));

    Ok(Html(html))
}