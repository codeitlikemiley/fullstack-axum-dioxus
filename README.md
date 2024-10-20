# Dioxus Fullstack (Island Architecture)

## TODO :
- see if we can use rsx on `components` crate so we dont have to write web-sys to access DOM elements

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


## Set up Pages crate

1. cargo init --lib pages
2. cd pages
3. install dependencies

```sh
cargo add dioxus 
cargo adddioxus-ssr
cargo add --path ../db
```

4. create `src/layout.rs`

<details>
<summary>layout.rs</summary>

```rust
#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Layout(title: String, children: Element) -> Element {
    rsx!(
        head {
            title { "{title}" }
            meta { charset: "utf-8" }
            meta { "http-equiv": "X-UA-Compatible", content: "IE=edge" }
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1"
            }
        }
        body { {children} }
    )
}
```
</details>

5. create `src/users.rs`

<details>
<summary>users.rs</summary>

```rust
use crate::layout::Layout;
use db::User;
use dioxus::prelude::*;

// Define the properties for IndexPage
#[derive(Props, Clone, PartialEq)] // Add Clone and PartialEq here
pub struct IndexPageProps {
    pub users: Vec<User>,
}

// Define the IndexPage component
#[component]
pub fn IndexPage(props: IndexPageProps) -> Element {
    rsx! {
        Layout { title: "Users Table",
            table {
                thead {
                    tr {
                        th { "ID" }
                        th { "Email" }
                    }
                }
                tbody {
                    for user in props.users {
                        tr {
                            td {
                                strong { "{user.id}" }
                            }
                            td { "{user.email}" }
                        }
                    }
                }
            }
        }
    }
}
```
</details>

6. update `src/lib.rs`

<details> 

<summary>lib.rs</summary>

```rust
mod layout;
pub mod users;
use dioxus::prelude::*;

pub fn render(mut virtual_dom: VirtualDom) -> String {
    virtual_dom.rebuild_in_place();
    let html = dioxus_ssr::render(&virtual_dom);
    format!("<!DOCTYPE html><html lang='en'>{}</html>", html)
}
```
</details>

7. cd to `server` crate

8. update dependencies

```sh
cargo add dioxus
cargo add --path ../pages
```

9. update `main.rs`

<details>
<summary>main.rs</summary>

```rust
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
```
</details>

10. run the server

## Set up `assets` crate for static files

1. cargo init --lib assets
2. cd assets
3. mkdir images
4. create an `avatar.svg` file on `images` folder

<details>

<summary>avatar.svg</summary>

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
    <path fill="#fff" d="M256 512A256 256 0 1 0 256 0a256 256 0 1 0 0 512zM160 256c0-53 43-96 96-96h64c53 0 96-43 96-96s-43-96-96-96H160zm0 96c0 53 43 96 96 96h64c53 0 96-43 96-96s-43-96-96-96H160zm0 96c0 53 43 96 96 96h64c53 0 96-43 96-96s-43-96-96-96H160z"/>
</svg>
```
</details>

5. touch `build.rs`

6. update `build.rs`

<details>
<summary>build.rs</summary>

```rust
use ructe::{Result, Ructe};

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env().unwrap();
    let mut statics = ructe.statics().unwrap();
    statics.add_files("images").unwrap();
    ructe.compile_templates("images").unwrap();

    Ok(())
}
```

</details>

7. add dependencies

```sh
cargo add mime@0.3
cargo add --build ructe@0.17 --no-default-features -F mime03
```

8. update `lib.rs`

<details>
<summary>lib.rs</summary>

```rust
include!(concat!(env!("OUT_DIR"), "/templates.rs"));

pub use templates::statics as files;
```
</details>

9. cd to `server` crate

10. create `static_files.rs`

11. update `static_files.rs`

<details>
<summary>static_files.rs</summary>

```rust
use assets::templates::statics::StaticFile;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{header, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;

pub async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    if let Some(data) = StaticFile::get(path) {
        Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(data.mime.as_ref()).unwrap(),
            )
            .body(Body::from(data.content))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}
```
</details>

12. modify `main.rs` to add the new route for static files

<details>
<summary>main.rs</summary>

```rust
// load module
mod static_files;

let app = Router::new()
    .route("/", get(users))
    .route("/static/*path", get(static_files::static_path)) // add this line
    .layer(Extension(config))
    .layer(Extension(pool.clone()));
... 
```

13. cargo add --path ../assets

14. use the static files on `pages/src/users.rs`

<details>
<summary>users.rs</summary>

```rust
// use avatar
use assets::files::avatar_svg;

...

// access the static file
img {
    src: format!("/static/{}", avatar_svg.name),
    width: "16",
    height: "16"
}
```

15. run the server

## Set up Components crate

1. cargo init --lib components

2. add dependencies

```toml
dioxus = "0.5.6"
js-sys = "0.3.72"
wasm-bindgen = "0.2.93"
web-sys = { version = "0.3.72", features = ["Document", "Element", "HtmlElement", "Window", "console"] }
```

3. set the crate type to `cdylib` and `rlib`

4. add example code to test on `src/lib.rs`

<details>

<summary>lib.rs</summary>

```rust
use js_sys::Math;
use wasm_bindgen::prelude::*;
use web_sys::{console, window, Element};

#[wasm_bindgen]
pub fn say_hello() {
    let random_number = Math::random();
    let message = format!("Hello from Rust! Random number: {}", random_number);

    // Log to the browser console
    console::log_1(&"Logging to console from Rust!".into());
    console::log_1(&format!("Generated random number: {}", random_number).into());

    // Show alert
    web_sys::window()
        .unwrap()
        .alert_with_message(&message)
        .unwrap();
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Access the DOM window object
    let window = window().unwrap();
    let document = window.document().unwrap();

    // Get the button element by ID
    let button: Element = document.get_element_by_id("alert-btn").unwrap();

    // Set an event listener for the button click
    let closure = Closure::wrap(Box::new(move || {
        // Call the Rust function say_hello
        say_hello();
    }) as Box<dyn Fn()>);

    // Set an event listener for the button click
    button
        .dyn_ref::<web_sys::HtmlElement>()
        .unwrap()
        .set_onclick(Some(closure.as_ref().unchecked_ref()));

    // We need to keep the closure alive, so we store it in memory.
    closure.forget();

    Ok(())
}
```

</details>

4. cd to assets crate
5. create `js/pages/users` folder
6. go back to `components` crate
7. generate assets using command

```sh
wasm-pack build --target web --out-dir ../assets/js/pages/users
```

8. Use the generated asset on `pages/src/users.rs`


```rust
            script {
                r#type: "module",
                dangerous_inner_html: r#"
import init from '/static/components.js';
init();
"#
            }
```

### Feature gating for wasm components

we need to use feature gating to only include components that are needed

```sh
wasm-pack build --target web --out-dir ../assets/js/pages/${feature} --features ${feature}
```

e.g.

```toml
[features]
default = []
users = []
featurex = []

```

on rust code we can do 

```rust
#[cfg(feature = "feature1")]
#[wasm_bindgen]
fn some_function() {
    // Implementation for feature1
}
```

