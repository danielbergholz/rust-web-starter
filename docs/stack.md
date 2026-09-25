# The minimal stack for full-stack Rust web apps

What is the smallest set of libraries, and the one most used by the community,
to build a full-stack web app in Rust with server-rendered HTML and a vanilla
HTML/CSS/JS frontend? This document explains the choices behind this template.

Versions verified with `cargo add` on 2026-09-25.

---

## 1. The ecosystem

Rust has no dominant "Rails". The community builds apps by combining libraries
(crates), closer to Node + Express than to Next.js. Compared to JavaScript:

- **There is a well-established default stack**: tokio + axum + serde + sqlx +
  tracing show up in almost every project.
- **Crates are bigger and last longer.** There are fewer tiny packages and much
  less churn.
- **Many come from the same group (tokio-rs)** and fit together naturally:
  tokio, axum, tower, tracing.
- **The cost shows up at compile time**: every dependency makes builds slower.
  8 direct crates pull in ~150–200 transitive dependencies, which is normal.

### "Batteries-included" frameworks, for comparison

| Framework | Model | Best for |
|---|---|---|
| **Topcoat** (tokio-rs) | Server rendering plus light reactivity (JS generated from Rust, "shards") | CRUD with some interactivity. Still experimental (v0.9) |
| **Leptos** | Fine-grained reactivity, SSR with WASM hydration | Rich, SPA-like UIs |
| **Dioxus** | React-like (RSX), runs on web, desktop and mobile | One codebase for several targets |
| **Loco** | Rails-like on top of axum: ORM, migrations, jobs, auth | A complete backend, frontend is up to you |
| **axum + libraries** | You assemble the stack | The most common choice. Stable and flexible |

---

## 2. The minimal list (8 crates)

| # | Crate | Role |
|---|---|---|
| 1 | `tokio` | Async runtime. Everything runs on top of it |
| 2 | `axum` | HTTP server: routes, extractors (`Form`, `Path`, `Query`, `Json`) and responses |
| 3 | `serde` | Forms and query strings → structs, and structs → JSON |
| 4 | `sqlx` | Database: plain SQL, connection pool and migrations |
| 5 | `askama` | Server-rendered HTML templates, checked at compile time |
| 6 | `tower-http` | Serves `static/` (`ServeDir`) and logs every request (`TraceLayer`) |
| 7 | `tracing` | Emits logs (`info!`, `error!`, spans) |
| 8 | `tracing-subscriber` | Prints those logs, with the level set by `RUST_LOG` |

```toml
[dependencies]
askama = "0.16"
axum = "0.8"
serde = { version = "1", features = ["derive"] }
sqlx = { version = "0.9", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tower-http = { version = "0.7", features = ["fs", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

To add them all to a new project:

```sh
cargo add tokio --features rt-multi-thread,macros
cargo add axum askama tracing
cargo add serde --features derive
cargo add sqlx --features runtime-tokio,sqlite,migrate
cargo add tower-http --features fs,trace
cargo add tracing-subscriber --features env-filter
```

Notes:

- To switch databases, replace the `sqlite` feature of sqlx with `postgres` or
  `mysql`.
- The `trace` feature of `tower-http` is what makes every request (method,
  path, status and latency) show up in the logs.
- Without `tracing` + `tracing-subscriber`, a 500 error happens with no
  explanation in the terminal. That is why everyone uses them.

---

## 3. Rendering HTML on the server

The handler builds the HTML as a `String`, and axum sends it with
`Content-Type: text/html`. There are three families of template engines:

| Family | Crates | Pros | Cons |
|---|---|---|---|
| Template files compiled into the app | **askama** (Jinja syntax), sailfish (EJS/ERB syntax) | Errors show up at compile time, and rendering is very fast | Changing a template requires recompiling |
| Template files loaded at runtime | **minijinja**, tera (used by Loco), handlebars | Hot reload without recompiling | Errors only show up when rendering |
| HTML inside Rust, via macros | **maud**, hypertext | Everything is Rust, and components are plain functions | Does not look like real HTML |

All of them **escape HTML automatically**, which protects against XSS.

Recommendation: **askama**, the most common choice with axum.

---

## 4. Frontend

### 4.1 What Rust developers use

There is no survey that measures this. The picture below comes from projects,
framework templates and community discussions. There are roughly three groups:

1. **Rust as a JSON API + a JS/TS SPA** (React, Svelte or Vue with Vite).
   Probably the majority, especially at companies. The frontend lives entirely
   in the JS world.
2. **Server-rendered HTML + "hypermedia".** The group that grew the most in
   recent years, in Rust and in Go. This is the approach of this template.
3. **Frontend in Rust compiled to WASM** (Leptos, Dioxus, Yew). A smaller, more
   enthusiast group.

Group 2 usually has no frontend build step:

- **CSS**
  - **Tailwind**: the most common, via the standalone CLI (a single binary, no
    Node). Sometimes with **DaisyUI** on top, for ready-made components.
  - **Hand-written CSS**: also common. Modern CSS has nesting, variables and
    `:has()`.
  - **Classless frameworks** such as Pico.css and Simple.css, for prototypes.
- **Reactivity**
  - **htmx**: makes the request and swaps part of the page with the HTML the
    server returns. The name most associated with this stack.
  - **Alpine.js**: local state (menus, modals, "copy" buttons). Often paired
    with htmx.
  - **Datastar**: newer and growing fast. Does the job of htmx and Alpine in a
    single library, using server-sent events.
  - **Vanilla JS**: `fetch` + `querySelector` for small interactions.
- **Instant navigation**
  - **htmx's `hx-boost="true"`** on `<body>`: links and forms swap only the
    `<body>` via AJAX. The most common way.
  - **Turbo (Hotwire)**: the same idea, from the Rails world. Less common in
    Rust.
  - **Native browser features** (Speculation Rules + View Transitions): the
    newest trend. See section 4.2.
  - **instant.page**: a tiny script that prefetches on hover. The classic
    before Speculation Rules.
- **Serving the files**
  - `tower-http::ServeDir` serving `static/`.
  - Or embedding the files in the binary with `rust-embed`, to deploy a single
    executable.

An important factor: **a Rust server answers in 1 or 2 ms**. With small HTML
and cached CSS/JS, a plain full-page navigation already feels instant. Very
often you do not need a SPA.

### 4.2 Decision: a vanilla frontend

This template uses **no frontend libraries**:

| Piece | Choice |
|---|---|
| CSS | Hand-written modern CSS: variables, `@layer`, nesting and `@scope` (section 4.4) |
| JS | Vanilla JS, only where needed |
| Instant navigation | Speculation Rules + View Transitions, native to the browser |

**Vanilla JS**: the live search in `static/app.js` asks the server for an HTML
fragment (the same askama partial used by the full page) and swaps it in:

```js
for (const input of document.querySelectorAll("[data-search-url]")) {
  const target = document.querySelector(input.dataset.searchTarget);
  let timer;

  input.addEventListener("input", () => {
    clearTimeout(timer);

    // Wait until the user stops typing for 300ms.
    timer = setTimeout(async () => {
      const url = `${input.dataset.searchUrl}?q=${encodeURIComponent(input.value)}`;
      const response = await fetch(url);
      target.innerHTML = await response.text();
    }, 300);
  });
}
```

**Instant navigation without libraries**, in `base.html` and the CSS:

```html
<!-- Prerender the next page when the user is about to click a link -->
<script type="speculationrules">
  { "prerender": [{ "where": { "href_matches": "/*" }, "eagerness": "moderate" }] }
</script>
```

```css
/* Smooth cross-page transitions, like a SPA */
@view-transition {
  navigation: auto;
}
```

> **GET routes must not have side effects.** With Speculation Rules the
> browser may request a page before the user clicks. A GET route that writes
> data (for example, a link shortener counting clicks on `GET /{slug}`) would
> record false visits. Use `POST` for anything that writes, or exclude those
> URLs from the rules (`"not": { "href_matches": "/go/*" }`).

Browser support is not universal: Speculation Rules work in Chromium-based
browsers, and cross-document View Transitions in Chromium and recent Safari.
Other browsers simply navigate normally, without the effect.

**When to evolve:** if the app grows to many pages and interactions, adopt htmx
(with `hx-boost`) + Alpine.js, or Datastar. The same search in htmx needs no
hand-written JS:

```html
<input type="search" name="q"
       hx-get="/notes/search"
       hx-trigger="input changed delay:300ms"
       hx-target="#notes">

<ul id="notes">
  {% include "notes.html" %}
</ul>
```

### 4.3 Why this approach works well with AI coding agents

What makes a stack good for agents:

1. **Errors show up early, with clear messages.** An agent succeeds when it can
   make a mistake, see the error and fix it on its own. The Rust compiler
   covers types, `null` and unhandled errors. With askama, even a wrong field
   name in a template breaks the build.
2. **Results can be verified without a browser.** With HTML coming from the
   server, an agent can test routes, redirects, errors and search with `curl`.
   In a SPA the state lives in the browser, and verifying it requires browser
   automation.
3. **The libraries are well known to the model.** axum, sqlx and askama are
   well represented in training data. New libraries, or ones whose API changes
   often (Topcoat, Datastar, recent Leptos versions), make it more likely that
   an agent invents APIs. Even with well-known libraries, versions quoted from
   memory can be outdated.
4. **A change touches few places.** A route, a template and maybe a migration,
   in one project and one language. With API + SPA there are two projects and a
   contract between them.

| Approach | For agents | Why |
|---|---|---|
| axum + askama + sqlx + little JS | Best | The compiler covers almost everything, testable with curl, stable libraries |
| Rust API + React/TS SPA | Very good | React and TS are what models know best, but there are two projects and verification needs a browser |
| Leptos / Dioxus | Fair | End-to-end types, but the API changed between versions, macro errors are confusing, and hydration bugs are hard to diagnose |
| Topcoat / Datastar | Harder | Very new: the agent has to read docs and source code every time |

**The weak spot:** whatever lives outside Rust is not checked by the compiler.
JS, htmx attributes and Alpine expressions are strings: a wrong id does not
fail, it just does nothing. That is why the JS stays minimal, and interactions
are tested in a browser.

**Tips:**

- **An `AGENTS.md`** with the stack, versions and verification
  commands (`cargo check`, `cargo clippy`, `cargo test`). This template ships
  one.
- **The `sqlx::query!` macros**: they check SQL against the real database at
  compile time, so the compiler covers SQL too.
- **Tests**, even a few: they catch what compiles but is wrong.
- **`#[axum::debug_handler]`** (requires axum's `macros` feature): makes axum's
  handler errors readable.
- **Pinned versions in `Cargo.toml`**, and checking `cargo doc` or docs.rs
  before using an unfamiliar API.

### 4.4 CSS: Tailwind or vanilla CSS?

**From an AI agent's point of view**, Tailwind has two advantages over
"traditional" CSS:

- **Locality**: the style lives on the element itself. There is no need to
  invent class names or keep a template and a stylesheet in sync.
- **Small blast radius**: changing an element's classes affects only that
  element. In traditional CSS, changing a rule can affect any page that uses
  the selector, and the cascade and specificity cause effects in distant
  places. Traditional CSS also accumulates dead rules that an agent cannot
  safely delete.

Neither is checked by the compiler: a misspelled class raises no error, it just
does not apply. Visual verification needs a screenshot in both cases.

Tailwind's downsides: one more binary with a `--watch` process running, and
templates cluttered with classes, which are tiring for people to read.

**Modern CSS removes most of those disadvantages**, with nothing to install:

**1. `@scope`: rules that apply only inside a component.** Solves the blast
radius problem. Inside the scope you can target `a` and `p` directly, without
inventing names:

```css
@scope (.note) {
  :scope { display: flex; align-items: center; gap: 1rem; }
  a { color: var(--color-accent); font-weight: 600; }
  p { color: var(--color-muted); font-size: 0.875rem; }
}
```

A `<style>` with a selector-less `@scope` applies only to its parent element.
That lets you put the CSS inside the template, like scoped styles in Vue or
Svelte, with the same locality as Tailwind (`templates/home.html` does this):

```html
<section class="notes">
  <style>
    @scope {
      li { display: flex; gap: 1rem; }
      p { flex: 1; margin: 0; }
    }
  </style>

  <ul id="notes">{% include "notes.html" %}</ul>
</section>
```

Do not put the `<style>` inside a `{% for %}` loop, or it is repeated for every
item.

**2. `@layer`: you define the priority order.** A later layer always wins over
an earlier one, regardless of specificity. No more `!important` wars:

```css
@layer reset, base, components, utilities;

@layer components {
  .notes li button { color: var(--color-accent); }   /* specificity (0,1,2) */
}

@layer utilities {
  .danger { color: var(--color-danger); }   /* (0,1,0), but wins: later layer */
}
```

**3. Native nesting**: everything about a component in one block, easy to find
and delete.

```css
.new-note {
  display: flex;
  gap: var(--space-3);

  & button {
    background: var(--color-accent);
    &:hover { background: var(--color-accent-hover); }
  }
}
```

**4. Supporting features:**

- **Custom properties (variables)**: play the role of Tailwind's design scale
  (colors, spacing).
- **`:where()`**: a selector with zero specificity, for base styles that are
  easy to override.
- **Container queries** (`@container`): a component adapts to the size of its
  container, not the whole viewport.

**Support**: nesting, `@layer`, `:where()`, variables and container queries
have worked in all major browsers for years. `@scope` is the newest (Firefox
was the last to ship it); check <https://caniuse.com/css-cascade-scope>.

**Conclusion:** with `@scope`, `@layer`, nesting and variables, vanilla CSS is
practically tied with Tailwind, for agents too:

- **Solved**: blast radius (`@scope`), specificity wars (`@layer`) and
  consistent values (variables).
- **Same in both**: a misspelled class raises no error, and visual verification
  still needs a screenshot.

Rules to keep vanilla CSS healthy:

- Every color and spacing value comes from a variable. No raw values.
- Component styles go inside `@scope`, preferably in the component's template.
- Layers are declared at the top: `@layer reset, base, components, utilities;`.

Consider switching to Tailwind if the app grows a lot and CSS changes start
breaking other pages.

---

## 5. The code in this repository

A small notes app that uses all 8 crates and every idea above:

```text
.
├── Cargo.toml
├── build.rs                     # rebuilds when a migration changes
├── migrations/
│   └── 0001_create_notes.sql
├── src/
│   └── main.rs                  # setup, routes, handlers, validation, errors, tests
├── static/
│   ├── app.js                   # live search (vanilla JS)
│   └── style.css                # tokens, @layer, nesting, dark mode, view transitions
└── templates/
    ├── base.html                # layout + Speculation Rules
    ├── home.html                # page, with component styles in @scope
    └── notes.html               # list fragment, also returned by /notes/search
```

Patterns worth copying:

- **`app(db) -> Router`** builds the whole router in one place.
- **`AppError`** turns any unexpected error into a logged 500. Handlers just
  use `?`.
- **`error_page`** re-renders the page with a validation message and a 400,
  instead of a bare error page.
- **Post/Redirect/Get** after every successful form submission.
- **Fragments**: `notes.html` is included by the full page and also rendered on
  its own by `/notes/search`, so the live search reuses the same markup.
- **Configuration by environment variables**: `DATABASE_URL`, `ADDR` and
  `RUST_LOG`, with sensible defaults.

---

## 6. Common alternatives for each piece

| Piece | Default | Alternatives |
|---|---|---|
| HTTP framework | axum | actix-web (also great; axum won by fitting the tokio/tower ecosystem) |
| Database | sqlx (plain SQL) | diesel or sea-orm (full ORMs), Toasty (tokio-rs ORM, still new) |
| Templates | askama | minijinja/tera (hot reload), maud (HTML in Rust) |
| Errors | a custom type (like `AppError`) | `anyhow` to simplify, `thiserror` for typed errors |

## 7. Next steps (beyond the minimum)

- **Input validation with dedicated crates**: for example, `url` to accept only
  `http`/`https` URLs. Without that, a link shortener would accept
  `javascript:...`.
- **Sessions and auth**: `tower-sessions`, `axum-login`.
- **Compile-time checked queries**: the `sqlx::query!` and `sqlx::query_as!`
  macros (they need `DATABASE_URL` at build time, or sqlx's offline mode).
- **Single-binary deploys**: `rust-embed` to embed `static/` in the
  executable.
- **Graceful shutdown**: `axum::serve(...).with_graceful_shutdown(...)` (needs
  tokio's `signal` feature).
