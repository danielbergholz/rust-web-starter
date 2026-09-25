# CLAUDE.md

Guidance for AI coding agents working in this repository. `AGENTS.md` is a
symlink to this file.

## Stack

Server-rendered web app. No frontend framework, no Node, no frontend build step.

| Layer | Choice |
|---|---|
| Runtime | `tokio` |
| HTTP | `axum` (routes, extractors, responses) |
| (De)serialization | `serde` (forms, query strings) |
| Database | `sqlx` with SQLite, plain SQL, migrations in `migrations/` |
| HTML | `askama` templates in `templates/`, checked at compile time |
| Static files and request logs | `tower-http` (`ServeDir`, `TraceLayer`) |
| Logging | `tracing` + `tracing-subscriber` |
| CSS | Modern vanilla CSS: tokens, `@layer`, nesting, `@scope` |
| JS | Vanilla JS, only where needed, in `static/app.js` |

Do not add dependencies (crates, JS libraries, CSS frameworks) without asking.

## Commands

```sh
cargo run                              # run the app on http://127.0.0.1:3000
cargo check                            # fast compile check
cargo fmt                              # format
cargo clippy --all-targets -- -D warnings
cargo test
```

Before finishing any task, run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`
and `cargo test`. All three must pass (CI runs the same commands).

Environment variables: `DATABASE_URL` (default `sqlite:app.db`), `ADDR` (default
`127.0.0.1:3000`), `RUST_LOG` (default `info,tower_http=debug`).

## Verifying changes

- The server returns plain HTML, so verify routes with `curl` (status codes,
  redirects, rendered HTML). If port 3000 is taken, run with
  `ADDR=127.0.0.1:3001 cargo run`.
- The compiler does not check CSS class names, JS, or `data-*` attributes. After
  changing those, check the page in a browser.

## Conventions

### Rust

- Handlers return `Result<_, AppError>`. Unexpected errors become a logged 500.
- Validation errors re-render the page with a message and a 400 (see
  `error_page`). Always validate on the server; browser checks are optional UX.
- Forms use Post/Redirect/Get: a successful `POST` answers with a redirect.
- **GET routes must not have side effects.** `templates/base.html` prerenders
  same-origin links (Speculation Rules), so the browser may request a GET
  route before the user clicks. Anything that writes data uses `POST`.
- Put pure logic (validation, parsing) in plain functions and unit test them in
  the `tests` module.

### Database

- Schema changes go in a **new** file in `migrations/` (`NNNN_description.sql`).
  Never edit a migration that has already been applied.
- Migrations run automatically on startup. `build.rs` makes Cargo rebuild when
  a migration changes.
- Always bind values with `.bind(...)`. Never build SQL with `format!` from user
  input.

### Templates

- Pages extend `base.html`. Fragments that are also returned on their own (for
  live updates) are separate templates, included with `{% include %}`.
- askama escapes all output by default. Do not use the `safe` filter on user
  data.

### CSS

- Global styles live in `static/style.css`, inside the layers declared at the
  top: `reset, base, components, utilities`.
- Use the design tokens in `:root` (`--color-*`, `--space-*`, `--text-*`,
  `--radius*`). Do not hardcode colors or spacing.
- Styles that belong to a single template go in that template, inside a
  `<style>@scope { ... }</style>` placed in the component's root element (never
  inside a `{% for %}` loop).
- Every color token has a dark mode value. When adding one, add both.

### JavaScript

- Keep it minimal and progressive: pages must work without JS.
- Prefer the server rendering HTML fragments over building HTML in JS (see the
  live search: `data-search-url` fetches a fragment and swaps it in).
