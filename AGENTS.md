# Repository Guidelines

## Project Overview

`hoshiyomi` is a Rust (edition 2024) single-crate web service: a MySQL-backed JSON REST API with user management, role-based access control (RBAC), dual authentication (server-side cookie sessions + JWT bearer with rotating refresh tokens), and machine-generated OpenAPI docs.

- **Stack**: axum 0.8 / tokio (full) / sqlx 0.9 (MySQL) / argon2 + jsonwebtoken / validator / utoipa 5 (Scalar + Swagger UI) / clap 4 + inquire / tracing.
- **Two targets**: library crate (`src/lib.rs`, exposes `build_app`, `config::{AppConfig, RawAppConfig}`, `state::AppState`) + binary `hoshiyomi` (`src/main.rs` → `cli::run()`).
- **CLI-first entry**: running `hoshiyomi` with no subcommand starts the HTTP server. Admin subcommands (`init`, `create-superuser`, `role list|create|delete`, `perms`, `config`) handle tasks deliberately not exposed over HTTP — role management is CLI-only.
- **Language convention**: all doc comments, inline rationale comments, and user-facing error messages are **Simplified Chinese** (e.g. `ApiResponse::error(404, "未找到")`).
- **API docs**: served at `/api-docs/scalar`, `/api-docs/swagger-ui`, `/api-docs/openapi.json` (utoipa-generated). There is no README; the OpenAPI spec is the API reference.

## Architecture & Data Flow

### Layered module structure

```
main.rs → cli::run → serve::serve → lib::build_app → modules::router
                                           ↓
                AppState { config, db, services }  ← DI container
                                           ↓
        modules/{user, role, auth}::{handlers → service → repository (free fns, &MySqlPool)}
```

- **Strict four-layer domain modules** (`user`, `auth`, `role`): `handlers.rs` (axum + utoipa, never touches SQL) → `service.rs` (business logic, concrete struct holding a `MySqlPool` clone) → `repository.rs` (**free async functions** taking `&MySqlPool` — no repository structs). `role` has models/repository/service only, no HTTP handlers.
- **Dependency direction**: `auth` → `user`/`role`, never upward; `state.rs` is the only place all three domains are wired together. `error.rs` is the leaf imported everywhere.
- **DI style**: no DI framework, no trait objects. One `AppState { config, db, services }` with accessors `cfg()/db()/srv()`; `Services` is a `#[derive(Clone)]` struct of five `#[derive(Clone)]` services (`UserService`, `RoleService`, `AuthService`, `SessionService`, `TokenService`), constructed once by `AppState::new`. The CLI builds `Services` directly (no `AppState`) for DB commands.

### Request lifecycle

1. `src/main.rs` → `cli::run().await` (`src/cli/run.rs`): `dotenvy::dotenv()` → clap parse; no subcommand → `serve::serve()`.
2. `src/serve.rs::serve`: `AppConfig::load()` → `infra::init_tracing` → `db::connect` → `db::migrate` (embedded migrations run at every startup) → `build_app(AppState::new(cfg, pool))` → bind → `axum::serve` with ctrl_c graceful shutdown.
3. `src/lib.rs::build_app`: `OpenApiRouter::new().nest("/api/v1", modules::router())` → merge Scalar/Swagger UIs → `.fallback(not_found)` **registered before layers** (deliberate — see comment: so 404s still pass CORS/Trace) → layer `middleware::session::refresh_session_cookie` → `TraceLayer` + CORS → `.with_state(state)`.
4. Per request: handler extracts `State<AppState>`, validated `AppJson/AppQuery/AppPath`, auth context `SessionCtx`/`JwtCtx`, calls `state.srv().<service>.<method>()`, returns `Result<impl IntoResponse, AppError>`.

### Auth model

- **Session flow**: `SessionCtx` extractor reads cookie → DB session lookup with expiry check. `middleware/session.rs` does sliding renewal post-response (extends expiry past half-TTL and re-sets cookie); it never makes auth decisions.
- **JWT flow**: `JwtCtx` (Bearer header, HS256), `TokenService::rotate_refresh_token` implements **single-use rotation**: fetch → DELETE → require `rows_affected == 1` (concurrent replay gets 0 → 401) → issue new pair. `TokenService` has a manual `Debug` impl redacting keys.
- **RBAC**: `Perm` enum in `src/modules/role/models.rs` (codes `*`, `user:read`, `user:*`, …) with `perms_match` wildcard logic; `roles.permissions` is a JSON column; `AuthService::require_permission` gates every protected handler.

## Key Directories

| Path | Purpose |
|---|---|
| `src/` | Library root; `build_app` router assembly |
| `src/config/` | Layered config (`mod.rs`, `schema.rs` serde structs + defaults, `paths.rs`, `meta.rs`) |
| `src/cli/` | clap CLI (`command.rs`), dispatch (`run.rs`), implementations (`command_impl.rs`) |
| `src/modules/{user,auth,role}/` | Domain modules; each: `mod.rs` (router), `models.rs`, `service.rs`, `repository.rs`, `handlers.rs` |
| `src/common/` | `response.rs` (`ApiResponse`, `PageData`), `extractor.rs` (validating `AppPath/AppQuery/AppJson`) |
| `src/middleware/` | `cors.rs`, `session.rs` |
| `src/util/` | `password.rs` (Argon2 helpers) |
| `migrations/` | sqlx migrations, compiled into binary, applied at serve startup |
| `tests/` | `api.rs` — black-box integration tests |

## Development Commands

```bash
cargo build                # debug build
cargo run                  # bin hoshiyomi; no subcommand = start HTTP server
cargo run -- init          # seed default roles (superuser/admin/user) — idempotent
cargo run -- create-superuser   # interactive (inquire) unless -u/-p/-e given
cargo test                 # integration + unit tests; no DB or env required
cargo fmt                  # pure defaults (no rustfmt.toml)
cargo clippy               # pure defaults (no clippy.toml)
```

- **Migrations**: embedded via `sqlx::migrate!("./migrations")`, run automatically when the server starts. **CLI subcommands do NOT migrate** — they assume the schema exists (created by first `serve`). No sqlx-cli needed.
- **Config precedence**: `RawAppConfig::default()` → optional `config.toml` → `HOSHIYOMI__`-prefixed env (separator `__`, e.g. `HOSHIYOMI__AUTH__JWT__SECRET`) → legacy flat env overrides (`DATABASE_URL`, `HOST`, `PORT`, `RUST_LOG`, `LOG_LEVEL`, `JWT_SECRET`, `JWT_EXPIRES_IN_SECONDS`, `SESSION_COOKIE_NAME`, `SESSION_TTL_HOURS`). `.env` loaded via dotenvy; all values have code defaults in `src/config/schema.rs`.
- **Local mode**: `HOSHIYOMI_LOCAL_MODE=1` (or an existing `./.hoshiyomi` dir) puts config/logs under `./.hoshiyomi` instead of the user config/data dirs.

## Code Conventions & Common Patterns

### Error handling — central, single-type

- One error type: `#[derive(Error)] AppError { kind: ErrorKind, message, errors, #[source] source }` (thiserror) with a separate 13-variant `ErrorKind` enum. `pub type Result<T> = std::result::Result<T, AppError>;` is the universal alias. Never introduce a second error type.
- Idioms: `ErrorKind::NotFound.msg("用户不存在")`; the `bail!(ErrorKind::NotFound, "...")` macro (single-arg form defaults to `Internal`); extension traits `ResultExt::err_kind[_msg]` and `OptionAppExt::ok_or_err_msg`; `register_errors!` macro generates `From` impls so `?` works directly on io/serde_json/config/sqlx/inquire/axum-rejection/validator/JoinError errors.
- **HTTP mapping** (`IntoResponse for AppError`): 400 validation/data-parse; 401 unauthorized; 403 forbidden/invalid-credentials/permission-denied; 404 not found; 409 already-exists; 500 everything else. Body is always the `ApiResponse` envelope with `code` = numeric HTTP status.
- **Anti-leak rule**: internal errors never expose source text to clients (`with_err` fills `message` from source only for non-internal kinds; `trace_source()` logs internals via `tracing::error!`). Keep this invariant.
- `map_duplicate_key(e, msg)`: converts MySQL duplicate-key error 1062 → `ErrorKind::AlreadyExists` (409) with a friendly Chinese message — use after uniqueness pre-checks as the race fallback (see `UserService::create`, `RoleService::{create, assign_to_user}`).
- **All user-facing messages are Simplified Chinese.**

### HTTP layer

- **Response envelope** (`src/common/response.rs`): `ApiResponse<T> { code, message, errors?, data }` — `code = 0` means success, otherwise the HTTP status; `errors` only present on validation failures. Use `ApiResponse::success/error/error_with_errors`, never bare JSON bodies. Pagination returns `PageData<T> { items, total, page, per_page }`.
- **Validating extractors**: use `AppPath<T>/AppQuery<T>/AppJson<T>` (not bare axum extractors) — they run `validator`'s `value.validate()` with `Rejection = AppError`. Request DTOs carry `#[validate(...)]` attributes with Chinese messages.
- **Auth extractors**: `SessionCtx` and `JwtCtx` implement `FromRequestParts<AppState>` with `Rejection = AppError`; `Option<SessionCtx>` is infallible for optional-auth routes.
- **Handlers**: return `Result<impl IntoResponse, AppError>`; every handler carries `#[utoipa::path(...)]` with Chinese descriptions; routers are `OpenApiRouter<AppState>` merged per module — keep the OpenAPI spec compiling.
- **Middleware ordering matters**: in `build_app`, the `not_found` fallback must stay registered before `.layer(...)` so 404s traverse CORS/Trace.

### Async & DB

- sqlx queries are runtime-checked raw string literals (`sqlx::query`/`query_as::<_, Row>` with `?` binds) — **not** compile-time `query!` macros (no build-time DATABASE_URL requirement). Keep it that way.
- Repositories: free `async fn` taking `&MySqlPool` first, ending `.fetch_optional(pool).await.map_err(AppError::from)`.
- DB pool: max 10 connections, `after_connect` pins session `time_zone = '+00:00'`; SQL writes use `UTC_TIMESTAMP()`, Rust reads use `DateTime<Utc>`. Pin timezone correctness when touching queries.
- Schema: utf8mb4/utf8mb4_unicode_ci; **no FOREIGN KEY constraints anywhere** — referential integrity is application-level.

### Naming & organization

- DTOs: `*Req` (requests, e.g. `RegisterReq`), `*Resp` (responses, e.g. `UserResp`), `*Row` (raw sqlx rows, e.g. `SessionRow`), domain rows bare (`User`, `Role`); services `*Service`; auth contexts `*Ctx`.
- `User` (FromRow) is deliberately **not** `Serialize` — password never leaves the server; responses go through `UserResp` via `From<User>`.
- Domain modules named singular nouns (`user`, `role`, `auth`); per-directory `mod.rs` with `pub mod` decls + re-exports.
- Config keys kebab-case; serde defaults kebab-case with explicit `rename` only where needed.
- Unit tests are inline `#[cfg(test)] mod tests` in the same file; integration tests in `tests/`.

## Important Files

| File | Role |
|---|---|
| `src/main.rs` | Binary entry → `cli::run()` |
| `src/lib.rs` | `build_app(AppState) -> Router`; public API surface |
| `src/serve.rs` | Server bootstrap: config → tracing → DB connect + migrate → bind |
| `src/error.rs` | `AppError`/`ErrorKind`/`bail!`/`Result`/`ResultExt`/`map_duplicate_key` — the error system core |
| `src/state.rs` | `AppState` + `Services` DI container |
| `src/config/schema.rs` | `RawAppConfig` + all defaults (port 8080, session TTL 24h, JWT TTL 900s, MySQL URL) |
| `src/config/mod.rs` | `AppConfig` (`Arc`-backed, cheap clone), `load()`/`load_raw()` layered resolution |
| `src/modules/auth/extractor.rs` | `SessionCtx`/`JwtCtx` auth contexts |
| `src/modules/auth/token.rs` | JWT + refresh-token single-use rotation |
| `src/modules/role/models.rs` | `Perm` vocabulary + `DefaultRole` seeding table |
| `src/db.rs` | `connect` (UTC pool), `migrate` (`sqlx::migrate!`) |
| `migrations/*.sql` | Schema, format `{YYYYMMDDHHMMSS}_{description}.sql` |
| `tests/api.rs` | Integration tests |
| `.env.example` | Documented env template (defaults live in `src/config/schema.rs`) |

## Runtime/Tooling Preferences

- **Runtime**: Rust — no rust-toolchain file, no `rust-version` declared; **effective MSRV is Rust ≥ 1.94.0** (sqlx 0.9.0 declares it). Edition 2024.
- **Package manager**: Cargo (lockfile v4); all deps from crates.io, no vendoring, no git/path deps. No workspace — single crate at repo root. No `[features]`.
- **Tooling**: no CI config, no rustfmt/clippy/deny configs — use `cargo fmt`/`cargo clippy` defaults. No Makefile/justfile.
- **Runtime requirements**: MySQL reachable at `DATABASE_URL` (default `mysql://hoshiyomi:password@127.0.0.1:3306/hoshiyomi`). Logs: stdout + daily-rotating JSON `access.log` under `Paths::log_dir()`; `RUST_LOG` controls the env filter.
- **OpenAPI**: utoipa 5 with `utoipa_axum::routes!` — adding an endpoint means adding its `#[utoipa::path]` docs or the spec breaks.

## Testing & QA

- **Framework**: built-in `cargo test` (tokio tests). Only dev-dependency: `http-body-util`. 8 tests total: 5 integration + 3 inline unit tests.
- **Integration style** (`tests/api.rs`): black-box — `build_app(AppState::new(AppConfig::new(RawAppConfig::default()), pool))` with a **`connect_lazy` MySQL pool pointed at an unreachable address**; requests dispatched in-process via `tower::ServiceExt::oneshot` (no TCP server). No config file/env reads; tests exercising DB paths assert the 500 failure path. Helpers `lazy_state/send/get_json/post_json` live in this file.
- **Unit tests**: synchronous, inline in `src/util/password.rs` (Argon2 round-trip) and `src/modules/role/models.rs` (permission code consistency, `perms_match` wildcard logic).
- **Coverage**: no coverage config, no CI. `cargo test` needs no DB, no env, no features.
- **Before touching auth/RBAC**: verify against `Perm` codes, `perms_match` wildcards, and `DEFAULT_ROLE_PERMISSIONS`; breakage there is silent at compile time (JSON column + runtime checks).
