# 🚀 Rust Playground

### Easy setup

#### Database

- Pull serlo.org repository
- Add a `docker-compose.override.yml` with:

```
version: '3.4'
services:
  mysql:
    ports:
      - '3306:3306'
```

- Use `yarn start:server` to start a local database with test data available at `mysql://root:secret@localhost:3306/serlo`

#### Rust

- Install rust https://www.rust-lang.org/tools/install
- Pull this repo
- Use `cargo run` to install dependencies and run the database layer
- Get your favourite article at `http://localhost:8080/uuid/1565`
