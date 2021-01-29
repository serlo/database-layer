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

Run `cargo run` to install dependencies and start the webserver. Now open [http://localhost:8080](http://localhost:8080). Happy coding!
