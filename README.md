<img src="https://raw.githubusercontent.com/serlo/frontend/staging/public/_assets/img/serlo-logo-gh.svg" alt="Serlo Logo" title="Serlo" align="right" height="75" />

# serlo.org – Database Layer

The database layer provides a Restful API in front of the database of [Serlo](https://serlo.org/).

## Setup

You need [Docker](https://docs.docker.com/engine/installation/) and [Rust](https://www.rust-lang.org) installed on your system. In order to run the pact tests (= contract test suite we use) you need to install [Node.js](https://nodejs.org/) version `14.x` and [yarn](https://yarnpkg.com/).

### Clone

```sh
# Clone the project:
$ git clone https://github.com/serlo/serlo.org-database-layer.git
$ cd serlo.org-database-layer
```

### Install sqlx-cli

You need to install [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) via

```sh
cargo install sqlx-cli
```

### Install dependencies for yarn and contract tests

Run `yarn` to install all necessary node dependencies (needed for development and running the contract tests).

### Install `jq` for `yarn fetch`

For the command `yarn fetch` the tool [`jq`](https://stedolan.github.io/jq/) needs to be installed.

### Install gcc and gcc-multilib

On Ubuntu you also need [`gcc`](https://gcc.gnu.org/) to run cargo.

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

- Run `yarn start` to start the database. It will be available under `mysql://root:secret@localhost:3306/serlo`.

## Development

Run `cargo run` to install dependencies and start the webserver.

Now you can mock a request from the terminal:

```sh
curl -H "Content-Type: application/json" -X POST -d '{"type":"UuidQuery","payload":{"id":1565}}' http://localhost:8080/
```

Happy coding!

### sqlx and `yarn sqlx:prepare`

We use [sqlx](https://github.com/launchbadge/sqlx) for creating and executing SQL queries. Here it is necessary that you run locally a local database (see section above) in order to be able to compile the source code. Also in the end of each PR the command `yarn sqlx:prepare` needs to be executed and the changes in [`sqlx-data.json`](./sqlx-data.json) need to be commited.

### Run all checks

The command `yarn check:all` will run all checks (like `cargo test` or `cargo clippy`) against the codebase. When you run `yarn check:all --no-uncommitted-changes` there is also a check whether you have uncommitted changes in your workspace. With this command you can test whether you are ready to open a pull request.

In case you want to run all tests automatically before pushing you can use a [git hook](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks). Here you can execute in a shell in the directory of this repository:

```sh
echo "yarn check:all --no-uncommitted-changes" > .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

### Test endpoints with `yarn fetch`

With `yarn fetch` you can test queries against the database layer. The script will compile and run a local version of the database layer if necessary. You can call it via

```sh
yarn fetch [Message] [Payload]
```

Example:

```sh
yarn fetch UuidQuery '{"id": 1}'
```

You can also omit the second argument if the endpoint does not need a payload:

```sh
yarn fetch SubjectsQuery
```

### Run contract tests

In order to run contract tests you need to start the server of the database layer via `cargo run` in a shell. Afterwards you can execute the contract tests with `yarn pacts`. There is also the script [`./scripts/pacts.sh`](./scripts/pacts.sh`) which automatically compiles and runs the server if necessary before running the contract tests.

You can also provide the path to a local pact file in case you want to test new API changes against the database layer. Example:

```sh
PACT_FILE=../api.serlo.org/pacts/api.serlo.org-serlo.org-database-layer.json ./scripts/pacts.sh
```

### Other helpful commands

- `cargo test` – Run all tests (see https://doc.rust-lang.org/book/ch11-01-writing-tests.html )
- `cargo clippy` – Lint the whole codebase (see https://github.com/rust-lang/rust-clippy )
- `yarn mysql` – Start a MySQL shell for the local mysql server.
- `yarn format` – Format all local source files.
- `yarn version` – Start process for adding new server version

- See also [`package.json`](./package.json) for the list of all yarn scripts.
