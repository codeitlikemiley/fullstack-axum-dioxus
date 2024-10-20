## Requirements

- geni cli
- dx cli
- wasm-bindgen-cli
- wasm-pack

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