# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Larust is an MVC starter-kit template combining Rust (Axum + Tokio) with a Laravel-inspired directory structure. It's a GitHub template repo, not a blank scaffold — it ships a working demo (cookie-session auth, user CRUD, file uploads) plus `docs/` walkthroughs showing the intended conventions for extending it (adding controllers, models, migrations, views).

Stack: Axum (web framework) + Tokio (async runtime) + Askama (compile-time type-checked HTML templates) + SQLx `AnyPool` (DB-agnostic SQLite/Postgres) + HTMX + Tailwind CSS v4 + bcrypt (password hashing) + axum-extra cookie jar (sessions).

## Commands

Local dev requires Node (for the Tailwind CLI) alongside Cargo.

```bash
# One-time setup
cargo install cargo-watch          # for hot reload
npm install                        # tailwindcss, concurrently

# Full dev loop: Tailwind watcher + cargo-watch server, in parallel
npm run dev

# Individual pieces
npm run dev:css       # tailwindcss --watch only
npm run dev:server    # cargo watch -i "static/*" -i "target/*" -x run
cargo run             # server only, no watch, no CSS build

# Production build (single self-contained binary, views precompiled in)
cargo build --release
npm run build:css     # minified Tailwind output

# Checks
cargo check
cargo build

# Tests (integration tests spin up the real router against an in-memory SQLite DB)
cargo test
cargo test some_test_name    # run a single test by name substring

# Scaffolding (see "CLI de scaffolding" below)
cargo make:model Product
cargo make:controller Product
cargo make:migration create_products_table
```

### Database / migrations

- Connection string lives in `.env` (`DATABASE_URL`), copy from `.env.example`. Default is `sqlite://database.sqlite` (the file is gitignored — it's created and migrated automatically on first run).
- Migrations run **automatically on server start** via `sqlx::migrate!("./migrations").run(&pool)` in `src/main.rs` — no manual step needed in dev. There is a single `migrations/` directory (not split per engine) because primary keys are app-generated UUIDv7 text, not DB autoincrement — see below.
- SQL files live in `migrations/`, named `<timestamp>_description.sql`. Manual control (optional): `cargo install sqlx-cli --no-default-features --features native-tls,sqlite,postgres`, then `sqlx migrate add <name>` / `sqlx migrate run`.
- **Primary keys are UUIDv7 text (`VARCHAR(36)`), generated in Rust via `db::new_id()`, not DB autoincrement.** SQLite's `AUTOINCREMENT` and Postgres's `SERIAL`/`GENERATED ALWAYS AS IDENTITY` are different syntax with different integer widths, so a `VARCHAR(36) PRIMARY KEY` populated app-side is what actually lets one migration file work unmodified on both engines. UUIDv7 (vs. v4) keeps IDs roughly time-ordered. Every insert must generate its own id and bind it explicitly (see `user_controller::create`, `auth_controller::register`, `upload_controller::store`).
- **`sqlx::Any` does not translate bind placeholders.** Despite the driver's "agnostic" name, it does **not** rewrite `?` to `$1, $2, ...` for Postgres — SQLite accepts `?` natively, Postgres rejects it with a syntax error (verified empirically against real sqlx 0.7 and 0.8 against a live Postgres instance; this is a real, non-obvious gap, not a config issue). `AppState::sql(query)` (`src/db.rs`) does that rewrite at runtime based on `AppState.is_postgres`. **Always write `?` in query strings and always pass them through `state.sql(...)`** before handing them to `sqlx::query`/`query_as` — skipping this silently breaks the query on Postgres only (SQLite dev testing won't catch it).
- **Any-driver decoding gotcha**: don't declare a `TIMESTAMP`/`DATETIME` column type in a migration if you plan to `query_as` it into a `String` — `sqlx::Any` can't map SQLite's `Datetime` type affinity and the query fails at runtime (not compile time). Use `VARCHAR`/`TEXT` instead (see `uploads.created_at`); `CURRENT_TIMESTAMP` still produces the same text value in both SQLite and Postgres.
- `main.rs` auto-creates the SQLite file if missing and seeds one `users` row (`info@larust.dev` / `password`) if the table is empty.

## Architecture

Request flow: `src/main.rs` (loads env, connects `AnyPool`, builds `AppState`, runs migrations/seed) → `larust::build_app` in `src/lib.rs` (assembles the full router) → `src/routes.rs` (route table, mounts the auth middleware on protected routes) → `src/controllers/*` (handlers, extract `State<AppState>`/`Path<String>`/`Form`/`Extension<User>`, query via SQLx, return an Askama template struct or a `Result<_, AppError>`) → `templates/*.html` (Askama, compiled into the binary) → response. If the request came from HTMX (e.g. the delete-user button), the handler returns a plain empty/fragment response instead of a full page.

**Binary + library split**: application code lives in `src/lib.rs` (`pub mod controllers/db/error/middleware/models/routes` plus `build_app(state: AppState) -> Router`); `src/main.rs` only does process bootstrap (env, DB connect, migrations, seed) and calls `larust::build_app`. This exists so `tests/integration_test.rs` can build the *exact* router the server runs, via `tower::ServiceExt::oneshot`, without duplicating router-assembly logic.

**`AppState`** (`src/db.rs`): `{ pool: AnyPool, is_postgres: bool }`, cloned into every handler via `State<AppState>` (Axum `Router<AppState>` throughout, not `Router<AnyPool>`). It exists solely to carry `is_postgres` alongside the pool so `state.sql(...)` can rewrite placeholders (see above). `db::new_id()` (UUIDv7 string) is the canonical way to mint a new row's primary key before inserting it.

**`Model`/`Insertable`** (`src/db.rs`): not an ORM (no relations, no query builder, no compile-time schema checking) — just enough to kill the repeated `SELECT/DELETE ... WHERE id = ?` boilerplate that used to be hand-written in every controller.
- `impl Model for X { const TABLE: &'static str = "xs"; }` is all a model needs to get `X::find(&state, id) -> Result<Option<X>, sqlx::Error>`, `X::all(&state) -> Result<Vec<X>, sqlx::Error>`, and `X::delete(&state, id) -> Result<(), sqlx::Error>` as default trait methods. These work for *any* struct because `#[derive(FromRow)]` maps by column name, so a generic `SELECT * FROM {TABLE}` always finds whatever fields the struct declares, in whatever order.
- `Insertable` (requires `Model`) adds generic `create`/`update`, but Rust has no reflection: a model must hand-write `fn fields(&self) -> Vec<(&'static str, db::Value)>` (column name → `Value::Str`/`OptStr`/`Int`) so the trait knows what to bind. See `src/models/upload.rs` for a real example — note `created_at` is deliberately left out of `fields()` so the migration's `DEFAULT CURRENT_TIMESTAMP` fills it instead.
- Only implement `Insertable` when a model's create/update is truly generic. `User` implements `Model` only (for `find`/`all`/`delete`) — its actual `create`/`update` stay hand-written in `user_controller.rs`/`auth_controller.rs` because they need bcrypt hashing and a friendly "email already taken" message on unique-constraint violation, which a generic trait method can't express.
- Both traits use native `async fn` in a trait (stable since Rust 1.75) rather than the `async-trait` crate; `#[allow(async_fn_in_trait)]` suppresses the associated lint because these traits are only ever used via static dispatch (`T: Model`), never `dyn Model` — verified this doesn't introduce a `Send`-future problem for Axum handlers before committing to the approach.
- `cargo make:model` emits the `impl Model for X { ... }` boilerplate automatically; it does not attempt to emit `Insertable` since that needs real column knowledge it doesn't have.

Conventions to follow when extending (mirrored in `docs/`, though the docs predate auth/uploads/error-handling and describe the simpler original shape):
- **Routes** (`src/routes.rs`): `web_routes(state)` returns public routes (`/`, `/login`, `/register`) merged with a `protected` sub-router (`/users*`, `/uploads`, `/logout`) that has `middleware::from_fn_with_state(state, require_auth)` applied via `.route_layer(...)`. `api_routes()` is mounted at `/api` for HTMX/JSON endpoints. Axum panics on two separate `.route()` calls for the same path — combine methods on one call (`get(x).post(y)`) instead of registering the path twice.
- **Controllers** (`src/controllers/`): one file per resource, declared in `controllers/mod.rs`. A handler returning HTML defines a `#[derive(Template)] #[template(path = "...")]` struct matching a file under `templates/` and returns it directly (askama_axum's blanket `IntoResponse` impl covers it — no explicit import needed at the call site). Handlers needing the logged-in user take `Extension<User>` (populated by the auth middleware, not re-queried). Fallible handlers return `Result<T, AppError>` and use `?` on `sqlx`/`io` calls rather than `unwrap_or_default()`. Path params for any resource are `Path<String>` (UUIDv7), not `Path<i64>`.
- **Models** (`src/models/`): plain structs deriving `FromRow` (+ `Serialize`/`Deserialize` as needed) that mirror a table's columns, since `query_as::<_, T>` maps by column **name** (verified empirically — `SELECT *` works regardless of column order). `id` fields are `String`. Re-exported through `models/mod.rs`. `User::password_hash` is `#[serde(skip_serializing)]` — never let it leak into a JSON response. Implement `db::Model` (just `const TABLE`) to get `find(state, id)` / `all(state)` / `delete(state, id)` for free — see `Model`/`Insertable` below.
- **Views** (`templates/`): Askama templates. `templates/layouts/base.html` defines `{% block %}` regions plus the top nav; feature templates `{% extends %}` it. Whitespace is minimized at compile time (`askama.toml`).
- **Errors** (`src/error.rs`): `AppError` (`NotFound` / `BadRequest` / `Internal`) implements `IntoResponse`, rendering `templates/errors/404.html` or `500.html`; `From<sqlx::Error>` and `From<std::io::Error>` log and convert to `Internal` so handlers can just use `?`. `error::not_found` is wired as the router's `.fallback(...)` for unmatched routes.
- **Auth** (`src/middleware/auth.rs`, `src/controllers/auth_controller.rs`): session token in an `HttpOnly` cookie (`session_id`, set via `axum-extra`'s `CookieJar`), backed by a `sessions` table (UUIDv7 id → `user_id`, `expires_at` as a unix timestamp — 7-day expiry). `require_auth` looks up the cookie, validates the session and user, and inserts `User` into request extensions on success; otherwise it redirects (303) to `/login`. Passwords are hashed with `bcrypt` (`User::hash_password` / `User::verify_password`); there's no password-reset flow.
- **File uploads** (`src/controllers/upload_controller.rs`): `axum::extract::Multipart` (needs the `multipart` feature on the `axum` dependency), saved to `static/uploads/<uuidv7>-<original-name>` and recorded in the `uploads` table (the same UUIDv7 is reused as both the row id and the filename prefix). The original filename is reduced to `Path::file_name()` before being used in the stored path — don't remove that, it's what prevents a crafted filename (e.g. `../../etc/passwd`) from writing outside the uploads directory.

`docs/` still has a per-concern deep-dive (`architecture.md`, `controllers.md`, `models.md`, `database.md`, `views.md`, `middleware.md`) for the original scaffolding patterns (routing, Askama, HTMX basics) — accurate for the mechanics, just not updated for the auth/upload/error-handling layer added on top.

## CLI de scaffolding (`cargo make:*`)

Three Cargo aliases (`.cargo/config.toml`, checked in — nothing to install) generate boilerplate for a new resource, Artisan-style:

- `cargo make:model <Nombre>` — writes `src/models/<snake>.rs` (a `FromRow`/`Serialize` struct with just `id: String`) and registers it in `src/models/mod.rs` (`pub mod` + `pub use`).
- `cargo make:controller <Nombre>` — writes `src/controllers/<snake>_controller.rs` (an `index` handler following the `State<AppState>` / `Result<T, AppError>` conventions above) **and** a matching `templates/<snake>s/index.html`, registers the controller in `src/controllers/mod.rs`, and prints the `.route(...)` line to paste into `src/routes.rs`.
- `cargo make:migration <description>` — writes `migrations/<timestamp>_<description>.sql` with the `VARCHAR(36) PRIMARY KEY` skeleton (see above). If `<description>` matches `create_X_table`, it infers `X` as the table name.

All three are implemented in `src/bin/larust-cli.rs` (a second binary in this package — `Cargo.toml` sets `default-run = "larust"` so plain `cargo run`/`cargo watch -x run` still resolve to the server, not the CLI) using simple string-template substitution (no templating crate). Generated code deliberately only references the `id` field so it compiles immediately; the TODO comments in the output mark where to add real columns. Routes are **never** auto-wired — that's left manual since it requires a judgment call (HTTP verb, whether the route needs `require_auth`).

## Notes

- `.gitignore` excludes `.env`, `/target`, `/Cargo.lock`, `database.sqlite`, `static/css/output.css` (compiled Tailwind, regenerated by `npm run dev:css`/`build:css`), and everything under `static/uploads/` except `.gitkeep`.
- README and all `docs/` content are written in Spanish; code comments and commit messages in this repo follow that convention too.
