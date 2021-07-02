<img src="https://assets.serlo.org/meta/logo.png" alt="Serlo logo" title="Serlo" align="right" height="60" />

# Serlo.org Database Layer

## Setup

You need [Docker](https://docs.docker.com/engine/installation/) and [Rust](https://www.rust-lang.org) installed on your system.

### Clone

```sh
# Clone the project:
$ git clone https://github.com/serlo/serlo.org-database-layer.git
$ cd serlo.org-database-layer
```

### Database

To get a local database for development, there are two approaches.

#### Via serlo.org

You can reuse the same database as a local serlo.org development environment. This is the recommended approach if you want to check whether the database layer has the same behavior as serlo.org.

- Setup [serlo.org](https://github.com/serlo/serlo.org)
- Add a `docker-compose.override.yml` file with the following content:
  ```yaml
  version: '3.4'
  services:
    mysql:
      ports:
        - '3306:3306'
  ```
- Run `yarn start` to start local serlo.org. The database will be available under `mysql://root:secret@localhost:3306/serlo`.

#### Without serlo.org

You can also use the database schema in this repository.

- Run `docker-compose up` to start the database. It will be available under `mysql://root:secret@localhost:3306/serlo`.

## Development

Run `cargo run` to install dependencies and start the webserver.

Now you can mock a request from the terminal:

```sh
curl -H "Content-Type: application/json" -X POST -d '{"type":"UuidQuery","payload":{"id":1565}}' http://localhost:8080/
```

Happy coding!

### Helpful commands

- `cargo test` – Run all tests (see https://doc.rust-lang.org/book/ch11-01-writing-tests.html )
- `cargo clippy` – Lint the whole codebase (see https://github.com/rust-lang/rust-clippy )

### Run all checks

The command `yarn check:all` will run all checks (like `cargo test` or `cargo clippy`) against the codebase. When you run `yarn check:all --no-uncommitted-changes` there is also a check whether you have uncommitted changes in your workspace. With this command you can test whether you are ready to open a pull request.

In case you want to run all tests automatically before pushing you can use a [git hook](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks). Here you can execute in a shell in the directory of this repository:

```sh
echo "yarn check:all --no-uncommitted-changes" > .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```
