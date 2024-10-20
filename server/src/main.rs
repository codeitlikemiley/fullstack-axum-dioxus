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

#[tokio::main]
async fn main() {
    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // build our application with a route
    let app = Router::new()
        .route("/", get(users))
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

pub async fn users(Extension(pool): Extension<db::Pool>) -> Result<Html<String>, CustomError> {
    let client = pool.get().await?;

    let users = db::queries::users::get_users().bind(&client).all().await?;

    let html = render(VirtualDom::new_with_props(
        IndexPage,
        IndexPageProps { users },
    ));

    Ok(Html(html))
}
