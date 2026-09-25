# rust-web-starter

A minimal template for full-stack web apps in Rust: server-rendered HTML, a
SQLite database, and a vanilla HTML/CSS/JS frontend. No Node, no frontend build
step, 8 crates.

Built to be easy to work on for both people and AI coding agents: the compiler
checks your Rust, SQL bindings and templates, and every route can be verified
with `curl`.

## Stack

| Crate | Role |
|---|---|
| [`tokio`](https://docs.rs/tokio) | Async runtime |
| [`axum`](https://docs.rs/axum) | HTTP server: routes, extractors, responses |
| [`serde`](https://docs.rs/serde) | Forms and query strings → structs |
| [`sqlx`](https://docs.rs/sqlx) | Database (SQLite), plain SQL, migrations |
| [`askama`](https://docs.rs/askama) | HTML templates checked at compile time |
| [`tower-http`](https://docs.rs/tower-http) | Static files and request logging |
| [`tracing`](https://docs.rs/tracing) + [`tracing-subscriber`](https://docs.rs/tracing-subscriber) | Logging |

Frontend: modern vanilla CSS (design tokens, `@layer`, nesting, `@scope`, dark
mode), a few lines of vanilla JS, and native instant navigation (Speculation
Rules + View Transitions).

**Why these choices?** See [docs/stack.md](docs/stack.md) for the full
reasoning, alternatives, and what the Rust community uses.

## Getting started

You need [Rust](https://rustup.rs). Create a new project from this template:

```sh
gh repo create my-app --template danielbergholz/rust-web-starter --private --clone
cd my-app
```

(Or click **Use this template** on GitHub.)

Rename the crate in `Cargo.toml` (`name = "my-app"`), then:

```sh
cargo run
```

Open <http://127.0.0.1:3000>. The SQLite database (`app.db`) is created and
migrated on startup.

## Commands

| Task | Command |
|---|---|
| Run the app | `cargo run` |
| Run with verbose logs | `RUST_LOG=debug cargo run` |
| Check that it compiles (fast) | `cargo check` |
| Format | `cargo fmt` |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| Test | `cargo test` |
| Browse the docs of every dependency | `cargo doc --open` |

Optional tools:

- [`bacon`](https://github.com/Canop/bacon): `cargo install bacon`, then
  `bacon run` rebuilds and restarts the app on every save.
- [`sqlx-cli`](https://crates.io/crates/sqlx-cli):
  `cargo install sqlx-cli --no-default-features --features sqlite`, then
  `sqlx migrate add <name>` creates a new migration.

## Configuration

| Variable | Default |
|---|---|
| `DATABASE_URL` | `sqlite:app.db` |
| `ADDR` | `127.0.0.1:3000` |
| `RUST_LOG` | `info,tower_http=debug` |

## Project structure

```text
├── build.rs              # rebuilds when a migration changes
├── migrations/           # SQL migrations, applied on startup
├── src/main.rs           # setup, routes, handlers, errors, tests
├── static/               # style.css and app.js, served at /static
├── templates/            # askama templates (base layout, pages, fragments)
├── docs/stack.md         # why this stack
└── AGENTS.md             # conventions for AI coding agents
```

The example app is a small notes list with create, delete and live search. It
shows the patterns to copy: a single `app()` router, an `AppError` type for
`?` in handlers, validation errors re-rendered with a 400, Post/Redirect/Get,
and HTML fragments shared by full pages and live updates.

## License

[MIT](LICENSE)
