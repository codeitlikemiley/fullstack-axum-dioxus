## Requirements

- geni cli
- dx cli
- wasm-bindgen-cli
- wasm-pack
- cargo-runner

## Initialize the app

1. mkdir hexagonal
2. ws init
3. add new members on `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = ["server", "components", "pages"]
```

4. copy [`.gitignore` ](https://gist.github.com/codeitlikemiley/f4b405d7afe8b76d7ce799c1732649db)
5. generate crates

```
cargo new server
cargo new components --lib
cargo new pages --lib
```

6. init

## Set migrations

1. export DATABASE_URL=postgres://postgres:postgres@localhost:5432/hexagonal

2. geni create

3. geni new create_users_table

4. geni up

## Add DB Queries

1. cargo init --lib db
2. cd db
3. mkdir queries
4. add initial queries

e.g. `users.sql`

```
--: User()

--! get_users : User
SELECT 
    id, 
    email
FROM users;
```

5. touch build.rs

<details> 
<summary>build.rs</summary>

```rust
use std::env;
use std::path::Path;

fn main() {
    // Compile our SQL
    cornucopia();
}

fn cornucopia() {
    // For the sake of simplicity, this example uses the defaults.
    let queries_path = "queries";

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let file_path = Path::new(&out_dir).join("cornucopia.rs");

    let db_url = env::var_os("DATABASE_URL").unwrap();

    // Rerun this build script if the queries or migrations change.
    println!("cargo:rerun-if-changed={queries_path}");

    // Call cornucopia. Use whatever CLI command you need.
    let output = std::process::Command::new("cornucopia")
        .arg("-q")
        .arg(queries_path)
        .arg("--serialize")
        .arg("-d")
        .arg(&file_path)
        .arg("live")
        .arg(db_url)
        .output()
        .unwrap();

    // If Cornucopia couldn't run properly, try to display the error.
    if !output.status.success() {
        panic!("{}", &std::str::from_utf8(&output.stderr).unwrap());
    }
}
```
</details>

6. Add `db` dependencies

```sh
cargo add cornucopia_async@0.6
cargo add tokio-postgres@0.7
cargo add deadpool-postgres@0.12
cargo add postgres-types@0.2
cargo add tokio@1 --features macros,rt-multi-thread
cargo add futures@0.3
cargo add serde@1 --features derive
```

7. modify `lib.rs`

<details>
<summary>lib.rs</summary>

```rust
use std::str::FromStr;

pub use cornucopia_async::Params;
pub use deadpool_postgres::{Pool, PoolError, Transaction};
pub use tokio_postgres::Error as TokioPostgresError;
pub use queries::users::User;

pub fn create_pool(database_url: &str) -> deadpool_postgres::Pool {
    let config = tokio_postgres::Config::from_str(database_url).unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    deadpool_postgres::Pool::builder(manager).build().unwrap()
}

include!(concat!(env!("OUT_DIR"), "/cornucopia.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn load_users() {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let pool = create_pool(&db_url);

        let client = pool.get().await.unwrap();
        //let transaction = client.transaction().await.unwrap();

        let users = crate::queries::users::get_users()
            .bind(&client)
            .all()
            .await
            .unwrap();

        dbg!(users);
    }
}
```
</details>

8. build the crate and run the tests using cargo-runner


## Set up server crate

1. cd server
2. touch src/config.rs

<details>
<summary>config.rs</summary>

```rust
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn new() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

        Config { database_url }
    }
}
```
</details>

3. touch src/errors.rs

<details>
<summary>errors.rs</summary>

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db::{PoolError, TokioPostgresError};
use std::fmt;

#[derive(Debug)]
pub enum CustomError {
    FaultySetup(String),
    Database(String),
}

// Allow the use of "{}" format specifier
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CustomError::FaultySetup(ref cause) => write!(f, "Setup Error: {}", cause),
            //CustomError::Unauthorized(ref cause) => write!(f, "Setup Error: {}", cause),
            CustomError::Database(ref cause) => {
                write!(f, "Database Error: {}", cause)
            }
        }
    }
}

// So that errors get printed to the browser?
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            CustomError::Database(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            CustomError::FaultySetup(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        };

        format!("status = {}, message = {}", status, error_message).into_response()
    }
}

impl From<axum::http::uri::InvalidUri> for CustomError {
    fn from(err: axum::http::uri::InvalidUri) -> CustomError {
        CustomError::FaultySetup(err.to_string())
    }
}

impl From<TokioPostgresError> for CustomError {
    fn from(err: TokioPostgresError) -> CustomError {
        CustomError::Database(err.to_string())
    }
}

impl From<PoolError> for CustomError {
    fn from(err: PoolError) -> CustomError {
        CustomError::Database(err.to_string())
    }
}
```
</details>

4. Add dependencies

```sh
cargo add axum@0.7 --no-default-features -F json,http1,tokio
cargo add tokio@1 --no-default-features -F macros,fs,rt-multi-thread
cargo add --path ../db
```

5. update `main.rs`

<details>
<summary>main.rs</summary>

```rust
mod config;
mod errors;

use crate::errors::CustomError;
use axum::{extract::Extension, response::Json, routing::get, Router};
use db::User;
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
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn users(Extension(pool): Extension<db::Pool>) -> Result<Json<Vec<User>>, CustomError> {
    let client = pool.get().await?;

    let users = db::queries::users::get_users().bind(&client).all().await?;

    Ok(Json(users))
}
```
</details>

6. run the server

