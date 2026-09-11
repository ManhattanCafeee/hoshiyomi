# Repository Guidelines

## Project Overview

`hoshiyomi` is a three-build-unit monorepo: a Rust/axum backend, a Nuxt 4 admin SPA (pnpm workspace with a generated alova SDK), and a uni-app mobile skeleton. The backend offers a MySQL-backed JSON REST API with user management, role-based access control (RBAC), dual authentication (server-side cookie sessions + JWT bearer with rotating refresh tokens), and machine-generated OpenAPI docs — the single source of truth for both frontend API clients.

- `backend/` — Rust (edition 2024) single crate. Stack: axum 0.8 / tokio (full) / sqlx 0.9 (MySQL) / validator / utoipa 5 (Scalar + Swagger UI) / clap 4 + inquire / tracing, on top of the in-house **`vivarium-rs` 0.3.3** toolkit (error envelope, validating extractors, CRUD/Query/Update, sessions + JWT + refresh tokens + passwords, config hot reload, OpenAPI mount + schema-description localization). Two targets: library crate (`src/lib.rs`, exposes `build_app`, `api_document`, `config::{AppConfig, RawAppConfig}`, `state::AppState`) + binary `hoshiyomi` (`src/main.rs` → `cli::run()`).
- `frontend/` — standalone pnpm workspace: Nuxt 4 SPA admin (`apps/admin`, srcDir `app/`) + generated alova SDK (`packages/apisdk`) + vendored internal Nuxt modules `packages/nuxt-modules` (`@hoshiyomi/alova`, `@hoshiyomi/nuxt-infra`, `@hoshiyomi/util`, `@hoshiyomi/shadcn`, `@hoshiyomi/tailwindcss`, `@hoshiyomi/nuxt-color-mode` — inlined in-repo, no git submodules).
- `uniapp/` — independent pnpm uni-app (Vue 3) mobile skeleton: register / login (JWT) / workbench / profile. H5 for dev, mp-weixin build supported. Based on the vitesse-uni-app template (uni-helper toolchain).
- **Codegen chain**: backend utoipa annotations → `cargo run --example dump_openapi > docs/openapi.json` (committed) → `pnpm gen:api` in `frontend/` and `uniapp/` separately. Contract changes must regenerate both SDKs in the same commit.
- **CLI-first entry**: running `hoshiyomi` with no subcommand starts the HTTP server. Admin subcommands (`init`, `create-superuser`, `role list|create|delete`, `perms`, `config`) handle tasks deliberately not exposed over HTTP — role management is CLI-only (no role pages in any frontend).
- **Language convention**: all doc comments, inline rationale comments, user-facing error messages, and **frontend UI copy** are **Simplified Chinese** (e.g. `ApiResponse::error(404, "未找到")`; admin sidebar labels 仪表盘/用户管理/个人资料).
- **API docs**: served at `/api-docs/scalar`, `/api-docs/swagger-ui`, `/api-docs/openapi.json` (utoipa-generated). The OpenAPI spec is the API reference for frontends; per-unit READMEs cover setup.

## Architecture & Data Flow

### Backend (all paths relative to `backend/`)

Layered module structure:

```
main.rs → cli::run → serve::serve → lib::build_app → modules::router
                                           ↓
                AppState { config, db, services }  ← DI container
                                           ↓
        modules/{user, role, auth}::{handlers → service (calls vivarium CRUD/Query/Update directly)}
```

- **Domain modules** (`user`, `auth`, `role`): `handlers.rs` (axum + utoipa, never touches SQL) → `service.rs` (business logic, concrete struct holding a `MySqlPool` clone; data access goes straight through `vivarium-rs`'s `create`/`find_by_id`/`delete`/`Query`/`Update`). `role` has models/service only, no HTTP handlers.
- **Dependency direction**: `auth` → `user`/`role`, never upward; `state.rs` is the only place all three domains are wired together. The error type is the library's `vivarium_rs::ApiError` (re-exported as `crate::{ApiError, ErrorKind, Result}` from `lib.rs`), imported everywhere.
- **DI style**: no DI framework, no trait objects. One `AppState { config, db, services }` with accessors `cfg()/db()/srv()`; `Services` is `Clone` over `UserService`, `RoleService`, `AuthService`, the fixed `SessionAuth` layer, and an `Arc<ArcSwap<AuthRuntime>>` (JWT verifier + refresh-token manager) that config hot reload rebuilds. Constructed once by `AppState::new(cfg, pool)`; the CLI builds `Services` directly (no `AppState`) for DB commands.

Request lifecycle:

1. `src/main.rs` → `cli::run().await` (`src/cli/run.rs`): `dotenvy::dotenv()` → clap parse; no subcommand → `serve::serve()`.
2. `src/serve.rs::serve`: `AppConfig::load()` → `vivarium_rs::serve::telemetry::init` → `db::connect` → `db::migrate` (embedded migrations run at every startup) → `AppState::new(cfg, pool)` → register hot-reload handlers → `build_app(state)` → bind → `axum::serve` with ctrl_c graceful shutdown. `cfg.watch()` stays alive for the process.
3. `src/lib.rs::build_app`: `OpenApiRouter::new().nest("/api/v1", modules::router())` → `.with_state(state)` → merge `vivarium_rs::openapi::mount(_, "/api-docs", api_document())` (`/api-docs/{openapi.json,scalar,swagger-ui}`) → `.fallback(not_found)` **registered before layers** (deliberate — see comment: so 404s still pass CORS/Trace) → layer `vivarium_rs::session_layer` → `TraceLayer` + CORS.
4. Per request: handler extracts `State<AppState>`, validated `PathVarser/QueryVarser/Varser`, auth context `SessionCtx`/`JwtCtx`, calls `state.srv().<service>.<method>()`, returns `Result<impl IntoResponse, ApiError>`. Chinese texts for library-produced messages come from `texts::install_chinese_texts()` (installed first thing in `cli::run` and `build_app`).

Auth model:

- **Session flow**: `vivarium_rs::session_layer` resolves the cookie, looks the session up by its SHA-256 digest, injects `SessionCtx<u64>` into request extensions, deletes dead rows and clears stale cookies; `SessionCtx` (app-side, `auth/extractor.rs`) only reads that extension. Sliding renewal at half-TTL happens post-response inside the same layer. A DB failure *during* the lookup surfaces as 401 `未授权` (the library layer swallows it into "no session"), not 500 — accepted library semantics.
- **JWT flow**: `JwtCtx` (Bearer header, HS256) decodes with `JwtVerifier` into `HsClaims { sub, username, exp, iat }`; `RefreshTokenManager::rotate` implements **single-use rotation** (hash → lookup → DELETE → require one row affected, so a replay gets 401) and re-signs the pair. Refresh tokens are stored as digests only.
- **RBAC**: `Perm` enum in `src/modules/role/models.rs` (codes `*`, `user:read`, `user:*`, …) with `perms_match` wildcard logic; `roles.permissions` is a JSON column; `AuthService::require_permission` gates every protected handler. Frontend permission checks mirror this: `*` and `prefix.*` wildcards in `usePermissions()` (admin) — no `super_admin` special-casing needed (superuser holds `*`).

### Admin frontend (session cookie flow)

```text
Page → Apis.<tag>.<OperationId>({ params, pathParams, data }).send()
  → alovaInstance (packages/apisdk/lib/api/index.ts): same-origin session Cookie, no auth headers
  → baseURL = runtimeConfig.public.apiBase ('') → dev Vite proxy /api/v1 + /api-docs → 127.0.0.1:8080
  → unwrap envelope {code:0(number), data} — code !== 0 → BizError → eventSystem 'request:error'
  → 401 → authAdapter plugin: toast「会话已过期」+ logout + /login (no token refresh — cookie TTL sliding renewal is server-side)
```

- Route guard: `definePageMeta({ auth: 'authenticated', permissions: ['user:read'] })` (login page `auth:'guest'` + `layout:false`); global `auth` middleware from `@hoshiyomi/nuxt-infra` (`packages/nuxt-modules/nuxt-infra/middleware/auth.ts`) — **modified for session auth: first client navigation `await adapter.init()` (GET /auth/me) before the auth decision** (no localStorage state).
- Login state: `useAuthState()` = `useState('auth')` memory-only `{user, permissions}` (Cookie is the only source of truth; `init()` re-fetches `/auth/me` on every page load). No pinia in admin.
- `/users` CRUD requires SessionCtx + `user:read/write/delete`; role management has no HTTP endpoint → no role page; menu built from `apps/admin/app/config/menu-registry.ts` with perm slugs.
- Generated SDK files (`packages/apisdk/lib/api/{apiDefinitions,createApis,globals}.ts`) are never hand-edited; `index.ts` is hand-written and preserved by `alova gen`.

### Uniapp (JWT flow)

```text
Page → Apis.auth.Auth__jwtLogin({ data }) (no .send(); alova method is thenable)
  → handwritten instance src/api/generated/index.ts (.handwritten is the source; gen:api restores it over index.ts)
  → baseURL = VITE_API_BASE_URL → adapter-uniapp → Bearer token from uni storage 'token' {userId, token, refreshToken, expiresAt}
  → unwrap {code:0(number), data} → code !== 0 → ApiError(code, message, data)
  → 401 → single-flight Apis.auth.Auth__jwtRefresh (refresh_token rotation) → replay once → failure: clear + reLaunch /pages/login/index
```

- File-based routing (`@uni-helper/vite-plugin-uni-pages`): every `.vue` under `src/pages/` is a route with `definePage({ layout, type, style })`; `pages.config.ts` = globalStyle + text-only tabBar (工作台/我的). `pages.json`/`manifest.json` are **generated and gitignored** — never hand-edit.
- Nav guard `src/router/interceptor.ts` (`uni.addInterceptor`); path constants `src/router/config.ts` (`PP.*` + `TAB_PATHS`) — never hardcode route strings.
- Pinia setup stores persisted via `uni.getStorageSync` keys `'token'`/`'user'` (`readPersisted()` factory); `isLoggedIn` checks token presence only — expiry handled by the 401 refresh path.
- Tab pages fetch data in `onShow`, not `onMounted`.

## Key Directories

| Path | Purpose |
|---|---|
| `backend/` | Rust backend crate (everything below this row with `src/`, `migrations/`, `tests/` is under `backend/`) |
| `backend/src/` | Library root; `build_app` router assembly |
| `backend/src/config/` | Layered config (`mod.rs` wrapper over `vivarium-config`, `schema.rs` serde structs + defaults, `legacy.rs` flat-env provider, `paths.rs`, `meta.rs`) |
| `backend/src/cli/` | clap CLI (`command.rs`), dispatch (`run.rs`), implementations (`command_impl.rs`) |
|`backend/src/modules/{user,auth,role}/`|Domain modules: `models.rs` + `service.rs` everywhere; `user`/`auth` add `mod.rs` (router) + `handlers.rs`, `auth` also `extractor.rs` (`SessionCtx`/`JwtCtx`) and `stores.rs` (SQL session/refresh-token stores); `role` has **no HTTP surface** (CLI-only)|
|`backend/src/pagination.rs`|App-side `PageData<T>` (u64 `total`/`page`/`per_page`) — the frozen wire type (`ApiResponse_PageData_UserResp`); the library's `Page<T>` stays internal|
| `backend/src/middleware/` | `cors.rs` only — session handling lives in the library's `session_layer` |
| `backend/examples/` | `dump_openapi.rs` — prints the OpenAPI JSON (no DB needed) |
| `backend/docs/` | `openapi.json` — committed codegen source |
| `backend/migrations/` | sqlx migrations, compiled into binary, applied at serve startup |
| `backend/tests/` | `api.rs` — black-box integration tests |
| `frontend/` | pnpm workspace root (install/lint/gen:api scripts; no dev/build scripts — those live in `apps/admin`) |
| `frontend/apps/admin/app/` | Nuxt srcDir: `pages/` (login, index, users, profile), `components/admin/`, `components/ui/` (shadcn-vue), `config/`, `plugins/`, `layouts/` |
| `frontend/packages/apisdk/` | Generated alova client (`lib/api/` — don't hand-edit except `index.ts`) + `useAuthState` store + composables/utils |
| `frontend/packages/nuxt-modules/` | Vendored internal Nuxt modules + `.prettierrc.json` (prettier config source) |
| `uniapp/` | Independent uni-app project: `src/pages/` (login/register/home/profile), `src/stores/`, `src/router/`, `src/api/generated/`, `src/style/tokens.scss` |
| `docker-compose.yml` | Local MySQL 8.4 (db/user `hoshiyomi`/`hoshiyomi`, password `password`) |

## Development Commands

### Backend (run in `backend/`)

```bash
cargo build                # debug build
cargo run                  # bin hoshiyomi; no subcommand = start HTTP server
cargo run -- init          # seed default roles (superuser/admin/user) — idempotent
cargo run -- create-superuser   # interactive (inquire) unless -u/-p/-e given
cargo test                 # integration + unit tests; no DB or env required
cargo fmt                  # pure defaults (no rustfmt.toml)
cargo clippy               # lint policy lives in the root Cargo.toml [workspace.lints]
cargo run -- example dump_openapi > docs/openapi.json   # export OpenAPI spec (SDK codegen source)
```

- **Migrations**: embedded via `sqlx::migrate!("./migrations")`, run automatically when the server starts. **CLI subcommands do NOT migrate** — they assume the schema exists (created by first `serve`). No sqlx-cli needed.
- **Config precedence**: `RawAppConfig::default()` → optional `config.toml` → `HOSHIYOMI__`-prefixed env (separator `__`, e.g. `HOSHIYOMI__AUTH__JWT__SECRET`) → legacy flat env overrides (`DATABASE_URL`, `HOST`, `PORT`, `RUST_LOG`, `LOG_LEVEL`, `JWT_SECRET`, `JWT_EXPIRES_IN_SECONDS`, `SESSION_COOKIE_NAME`, `SESSION_TTL_HOURS`). `.env` loaded via dotenvy; all values have code defaults in `backend/src/config/schema.rs`. Resolution runs through `vivarium-config` (figment), and `config.toml` is **watched**: `auth.jwt.*` edits take effect without a restart, while `server`/`database`/`log`/`auth.session` edits only log a `需重启生效` warning. Legacy flat vars are merged last, so **`.env` overrides `config.toml`** (e.g. `PORT=8080` in `.env` beats `[server] port` in the file).
- **Local mode**: `HOSHIYOMI_LOCAL_MODE=1` (or an existing `./.hoshiyomi` dir) puts config/logs under `./.hoshiyomi` instead of the user config/data dirs.

### Admin frontend (run in `frontend/`)

```bash
pnpm install               # triggers admin postinstall (nuxt prepare)
pnpm gen:api               # backend/docs/openapi.json → packages/apisdk/lib/api
pnpm --filter admin dev    # :3000, proxy /api/v1 + /api-docs → 127.0.0.1:8080
pnpm --filter admin build  # SPA build (ssr:false, prerender / and /login)
pnpm check                 # eslint + prettier (config packages/nuxt-modules/.prettierrc.json)
pnpm type-check            # --filter admin vue-tsc
```

### Uniapp (run in `uniapp/` — independent dir, own lockfile; never mix with frontend/)

```bash
pnpm install               # node-linker=hoisted + allowBuilds live in pnpm-workspace.yaml (pnpm 11)
pnpm gen:api               # backend/docs/openapi.json → src/api/generated, then restores .handwritten → index.ts
pnpm dev                   # H5 dev; pnpm dev wx for WeChat Mini Program
pnpm build                 # H5 build (also: pnpm build wx / pnpm build app)
pnpm type-check            # vue-tsc --noEmit
pnpm lint / pnpm lint:fix  # @uni-helper/eslint-config (ignores src/api/generated/**)
```

### Infra

```bash
docker compose up -d       # root: MySQL 8.4 :3306 (hoshiyomi/password, db hoshiyomi)
```

## Code Conventions & Common Patterns

### Backend — error handling, central single-type

- One error type, owned by the library: `vivarium_rs::ApiError { kind, message, errors, source }` with a 9-variant `ErrorKind` (`BadRequest`/`DataParse`/`Validation`/`Unauthorized`/`Forbidden`/`NotFound`/`Conflict`/`TooManyRequests`/`Internal`). `crate::Result<T>` is the universal alias. Never introduce a second error type.
- Idioms: `ApiError::not_found("用户不存在")` for the common kinds; `ApiError::new(ErrorKind::Internal, "…").with_source(e)` when the status kind and the client message differ; `?` works directly on io/serde_json/sqlx errors (library `From` impls — `sqlx::Error` maps `RowNotFound` → 404 and unique violations → 409). `config::ConfigError`, `inquire::InquireError` and `JoinError` have no library conversion: map them explicitly at the call site.
- **HTTP mapping**: `DataParse`/`BadRequest` 400, `Validation` **422**, `Unauthorized` 401, `Forbidden` 403, `NotFound` 404, `Conflict` 409, `TooManyRequests` 429, `Internal` 500. Body is always the `ApiResponse` envelope with `code` = numeric HTTP status.
- **Anti-leak rule**: internal errors never expose source text to clients (the client message is the `Texts` catalog entry); the source is kept for `tracing` and only echoed in a `system` field while `install_debug_mode` is on. Keep this invariant.
- Uniqueness races: rely on the library's `From<sqlx::Error>` (409) or `ApiError::conflict_from_db(e, msg)` when the interface's Chinese message must survive — after the explicit pre-check, as before (see `UserService::create`, `RoleService::{create, assign_to_user}`).
- Library-produced messages are English by default; `backend/src/texts.rs::install_chinese_texts()` injects the Chinese catalog, with `echo_details = true` so 4xx messages keep the underlying detail (`用户 id 无效: abc`). It must run before any other library call — hence the first line of `cli::run` and `build_app`. The same module's `localize_schema()` rewrites the library's English rustdoc *schema descriptions* to Chinese via `openapi::localize` (called from `lib.rs::set_info`, so `api_document()` and the served document both get it).

### Backend — HTTP layer

- **Response envelope** (library-owned, used directly as `vivarium_rs::ApiResponse`): `ApiResponse<T> { code, message, errors?, data }` — `code = 0` means success, otherwise the HTTP status; `errors` only present on validation failures; `data` always serialized (`null` on error). Success bodies use `ApiResponse::ok(data)`, never bare JSON; failures are `ApiError` (never hand-built envelopes). Pagination returns the app-side `PageData<T> { items, total, page, per_page }` (u64 — maps `ApiResponse_PageData_UserResp`), built from the library's `Page<User>` inside the service.
- **Validating extractors**: use `vivarium_rs::{PathVarser, QueryVarser, Varser}` (not bare axum extractors) — they run `Initializer::try_initialize` then `validator`'s `validate()`, with `Rejection = ApiError` (parse failure → 400, validation failure → 422). `Varser` consumes the body and must be the **last** parameter; `PathVarser`/`QueryVarser` are `FromRequestParts`. Every DTO used with them needs `impl vivarium_rs::Initializer for X {}`. Request DTOs carry `#[validate(...)]` attributes with Chinese messages.
- **Auth extractors**: `SessionCtx` and `JwtCtx` implement `FromRequestParts<AppState>` with `Rejection = ApiError`; `SessionCtx` only reads the extension the library's session layer injected.
- **Handlers**: return `Result<impl IntoResponse, ApiError>`; every handler carries `#[utoipa::path(...)]` with Chinese descriptions **and an explicit `operation_id` (`Auth__login`, `User__list`, …)** — the operation id is the generated SDK function name, change it only with a coordinated SDK regen; routers are `OpenApiRouter<AppState>` merged per module.
- **Middleware ordering matters**: in `build_app`, the `not_found` fallback must stay registered before `.layer(...)` so 404s traverse CORS/Trace.

### Backend — async & DB

- sqlx queries are runtime-checked raw string literals (`sqlx::query`/`query_as::<_, Row>` with `?` binds) — **not** compile-time `query!` macros (no build-time DATABASE_URL requirement). Keep it that way.
- Data access: services call the library's `create`/`find_by_id`/`delete`/`Query`/`Update` with `&pool` (row-returning calls additionally need `T: sqlx::FromRow + Send + Unpin`; `Query::paginate` needs a `Copy` executor, so pass `&pool`). Entities carry `#[derive(vivarium_rs::Entity)]` + a `*Col` `Column` enum; `created_at`/`updated_at` are `#[entity(skip)]` so writes keep the DB defaults. The two statements the library cannot express — the `user_roles × roles` JOIN and the `user_roles` INSERT — stay as raw `sqlx::query*` inside `role/service.rs`.
- DB pool: max 10 connections, `after_connect` pins session `time_zone = '+00:00'`; timestamp columns default to `UTC_TIMESTAMP()`, `Update::set_expr(…, Expr::Now)` renders `CURRENT_TIMESTAMP` (equal only because the session tz is pinned), Rust reads use `DateTime<Utc>`. Pin timezone correctness when touching queries.
- Schema: utf8mb4/utf8mb4_unicode_ci; **no FOREIGN KEY constraints anywhere** — referential integrity is application-level.

### Backend — naming & organization

- DTOs: `*Req` (requests, e.g. `RegisterReq`), `*Resp` (responses, e.g. `UserResp`), domain rows bare (`User`, `Role`); services `*Service`; auth contexts `*Ctx`.
- `User` (FromRow) is deliberately **not** `Serialize` — password never leaves the server; responses go through `UserResp` via `From<User>`.
- Domain modules named singular nouns (`user`, `role`, `auth`); per-directory `mod.rs` with `pub mod` decls + re-exports.
- Config keys kebab-case by default; the auth sections keep the library's explicit snake_case serde renames (`cookie_name`, `ttl_hours`, `expires_in_seconds`) because both `config.toml` and `HOSHIYOMI__AUTH__SESSION__COOKIE_NAME`-style env vars map through them.
- Unit tests are inline `#[cfg(test)] mod tests` in the same file; integration tests in `backend/tests/`.

### Admin frontend

- **API calls**: `Apis.<tag>.<OperationId>({ params, pathParams, data }).send()` → unwrapped business data. Lists: `useRequest(() => Apis.user.User__list({ params }), { immediate: false })`-style + `extractData`/`extractTotal` (`packages/apisdk/utils/pagination.ts`). Error toasts via `useNotify().apiError(err)` (extracts envelope `message`, Chinese).
- **Envelope**: alova instance unwraps `code === 0` (**number** — Rust i32; not the Go reference's string `'0'`) → `data`; failures throw `BizError` (from `@hoshiyomi/alova/lib`) and emit `eventSystem` events; `authAdapter.client.ts` subscribes `request:error` for 401.
- **No token refresh**: session Cookie + server-side sliding renewal; 401 → logout + redirect `/login`.
- **Permissions**: `usePermissions().hasPermission('user:read')` with `*`/`x.*` wildcards — gate both templates and scripts; `definePageMeta({ permissions: [...] })` for route-level 403.
- **Dialogs**: `props(record: X | null)` (null = create) + `defineModel<boolean>('open')` + `defineEmits<{ save: [data] }>` + `watch(open, resetForm)`; parent owns the API call + notify + refresh.
- **UI**: shadcn-vue (Reka UI) components under `components/ui/`, `cn()` from `app/lib/utils.ts`, Tailwind v4, `@lucide/vue` icons; all labels Simplified Chinese.
- **Generated files never hand-edited**; `packages/apisdk/lib/api/index.ts` is the hand-written instance (preserved by codegen). Hand-written domain types go in `lib/domain/models/` (auto-imported).

### Uniapp

- **API calls**: `Apis.auth.Auth__jwtLogin({ data })` (no `.send()` — uniapp call style, alova method is thenable). Manual imports only: `Apis` from `@/api/generated`, `PP` from `@/router/config`, pinia stores; everything else auto-imported (vue/uni-app/vueuse/composables/stores/components).
- **SFC rules**: `<script setup lang="ts">` only; tag order script→template→style; style always `lang="scss" scoped`; `definePage({ layout, type, style })` right after imports; page files kebab-case, components PascalCase, stores `useXxxStore` (feature-name file), CSS classes kebab-case.
- **Tab pages fetch in `onShow`** (not `onMounted`); errors → `uni.showToast({ icon: 'none', title: e.message })`.
- **API layer**: `src/api/generated/index.ts.handwritten` is the SOURCE of the alova instance — edit that file, `gen:api` restores it over `index.ts`. 401 → single-flight `Auth__jwtRefresh` replay; envelope `code !== 0` → `ApiError`.
- **Platform forks** (when real divergence appears): `// #ifdef H5` / `// #ifdef MP-WEIXIN` conditional compilation; keep both branches.
- **Styling**: SCSS tokens `$app-*` + mixins in `src/style/tokens.scss` (globally injected via vite `additionalData`); `rpx` units; UnoCSS `i-carbon-*` icons for one-off icons.

## Important Files

| File | Role |
|---|---|
| `backend/src/main.rs` | Binary entry → `cli::run()` |
| `Cargo.toml` | Root **virtual workspace** manifest (editor/tooling discovery; `default-members = ["backend"]`) |
| `backend/src/lib.rs` | `build_app(AppState) -> Router`; public API surface |
| `backend/src/serve.rs` | Server bootstrap: config → telemetry → DB connect + migrate → hot-reload registration → bind |
|`backend/src/texts.rs`|Chinese texts: `install_chinese_texts()` (library message catalog) + `localize_schema()` (library schema descriptions)|
|`backend/src/state.rs`|`AppState` + `Services` DI container + `AuthRuntime` snapshot (`ArcSwap`)|
| `backend/src/config/legacy.rs` | The 9 legacy flat env vars as a figment provider (highest priority) |
| `backend/src/config/schema.rs` | `RawAppConfig` + all defaults (port 8080, session TTL 24h, JWT TTL 900s, MySQL URL) |
| `backend/src/config/mod.rs` | `AppConfig` — thin wrapper over `vivarium_rs::Config<RawAppConfig>` (`load`/`get`/`watch`/`register`) |
| `backend/src/modules/auth/extractor.rs` | `SessionCtx`/`JwtCtx` auth contexts |
| `backend/src/modules/auth/stores.rs` | `SessionStore`/`RefreshTokenStore` MySQL impls (SHA-256 digest keys) |
| `backend/src/modules/role/models.rs` | `Perm` vocabulary + `DefaultRole` seeding table |
| `backend/src/db.rs` | `connect` (UTC pool), `migrate` (`sqlx::migrate!`) |
| `backend/examples/dump_openapi.rs` | Prints OpenAPI JSON (no DB); output → `backend/docs/openapi.json` |
| `backend/docs/openapi.json` | Committed codegen source for both frontends |
| `backend/migrations/*.sql` | Schema, format `{YYYYMMDDHHMMSS}_{description}.sql` |
| `backend/tests/api.rs` | Integration tests |
| `backend/.env.example` | Documented env template (defaults live in `src/config/schema.rs`) |
| `frontend/apps/admin/nuxt.config.ts` | SPA config, modules, `/api/v1` + `/api-docs` dev proxy, apiBase, **explicit `vite.server.fs.allow` for the workspace root** (monorepo module sources) |
| `frontend/alova.config.ts` | wormhole codegen config (no defaults/to plugins — see below) |
| `frontend/packages/apisdk/lib/api/index.ts` | Hand-written alovaInstance: envelope unwrap (`code === 0` number), no refresh, event system |
| `frontend/packages/apisdk/stores/authState.ts` | `useAuthState` — memory-only `{user, permissions}` |
| `frontend/packages/nuxt-modules/nuxt-infra/middleware/auth.ts` | Global auth middleware — `await adapter.init()` on first client nav (session flow) |
| `frontend/apps/admin/app/plugins/authAdapter.client.ts` | AuthAdapter impl: init via `Auth__me`, 401 → logout + redirect |
| `frontend/apps/admin/app/config/menu-registry.ts` | Sidebar menu single source of truth (perm slugs) |
| `uniapp/alova.config.ts` | uniapp codegen config (same input, `src/api/generated` output) |
| `uniapp/src/api/generated/index.ts.handwritten` | Hand-written uniapp alova instance (Bearer + 401 single-flight refresh) |
| `uniapp/src/router/{config,interceptor}.ts` | `PP.*` path constants + nav guard |
| `uniapp/src/stores/{auth,user}.ts` | Pinia stores, `uni.getStorageSync` keys `'token'`/`'user'` |
| `uniapp/pnpm-workspace.yaml` | `nodeLinker: hoisted` + overrides + allowBuilds (pnpm 11 settings; project `.npmrc` equivalents are ignored) |

## Runtime/Tooling Preferences

- **Rust**: no rust-toolchain file; `backend/Cargo.toml` declares `rust-version = "1.94"` (sqlx 0.9's floor) so `resolver = "3"` resolves MSRV-aware, and `publish = false` (private app — optional metadata lints do not apply). Edition 2024. Cargo lockfile v4 at the repo root; all deps from crates.io; single crate in `backend/`. The **root `Cargo.toml` is a virtual workspace manifest** (`members = ["backend"]`, `resolver = "3"`) so editors/rust-analyzer discover the crate from the repo root — build artifacts land in root `target/`. `cargo run` from root requires `-p hoshiyomi`; CLI flows are documented as `cd backend`.
- **Lint policy**: the root `Cargo.toml` carries `[workspace.lints.*]` — `unsafe_code = "forbid"`, `missing_docs = "warn"`, `clippy::all = { level = "deny", priority = -1 }` (the group needs a lower priority than the individual lints) plus three picked-from-pedantic lints (`uninlined_format_args`, `redundant_closure_for_method_calls`, `unused_async`); the member opts in with `[lints] workspace = true`, so plain `cargo clippy` (no flags) is the gate. `unused_async_trait_impl` is deliberately **not** enabled — axum's `FromRequestParts` impls must be `async` even when they await nothing. Library public surface is deliberately narrow: `cli`, `config`, `state` are `pub` (bin + integration tests); everything else is `pub(crate)`.
- **Node/pnpm**: Node ≥ 22 (26 verified locally); pnpm 11 (`frontend/` workspace, lockfile v9). `uniapp/` is an **independent** project: own lockfile, own `pnpm-workspace.yaml` carrying `nodeLinker: hoisted` (uni-app's build chain forces `preserveSymlinks: true`, so transitive deps must be flat at root — do NOT remove this), never mix pnpm invocations between the two dirs. No `packageManager` field, no corepack.
- **Backend tooling**: no rustfmt.toml/clippy.toml/deny.toml — `cargo fmt`/`cargo clippy` defaults, with the enforced lint set centralized in the root `[workspace.lints]`. No Makefile/justfile.
- **CI**: `.github/workflows/ci.yml` — three jobs: `backend` (fmt / clippy `-D warnings` / test / `docs/openapi.json` must equal `dump_openapi` output), `admin` and `uniapp` (regenerate the SDK and fail if it differs from the committed files, then type-check; admin also builds). Run the same commands locally — they need no database.
- **Frontend tooling**: prettier config at `frontend/packages/nuxt-modules/.prettierrc.json` (semi false, single quotes, width 120, trailing commas); eslint configs per unit (admin via `.nuxt`-generated config, uniapp via `@uni-helper/eslint-config`).
- **Backend runtime**: MySQL reachable at `DATABASE_URL` (default `mysql://hoshiyomi:password@127.0.0.1:3306/hoshiyomi`); `docker compose up -d` at root provides it. Logs: stdout + daily-rotating JSON `access.log` under `Paths::log_dir()`; `RUST_LOG` controls the env filter. No Redis.
- **OpenAPI**: utoipa 5 with `utoipa_axum::routes!` — adding an endpoint means adding its `#[utoipa::path]` (with `operation_id`) or the spec breaks. `lib.rs::set_info` is the shared finalizer (info + `texts::localize_schema`), and both `api_document()` and `build_app` derive their document from their own router's `split_for_parts()`, so the committed JSON and the served `/api-docs/openapi.json` cannot drift. Library-owned schema descriptions are English rustdoc; `localize_schema` maps them to Chinese through `openapi::localize` — **a library upgrade that adds or rewords a description needs a new table entry**, and `tests/api.rs::openapi_schema_descriptions_are_chinese` fails until it is added.
- **Codegen gotchas**: `alova gen` does NOT use `defaultsPlugin`/`toPlugin` (their output references response schemas that utoipa inlines into `ApiResponse_*` envelopes — broken types, unused code). Generated `defaults.ts`/`to.ts` must not exist; regenerate both SDKs when the contract changes; commit `backend/docs/openapi.json` with the change.

## Testing & QA

- **Backend framework**: built-in `cargo test` (tokio tests). Only dev-dependency: `http-body-util`. 12 tests total: 7 integration + 5 inline unit tests.
- **Backend integration style** (`backend/tests/api.rs`): black-box — `build_app(AppState::new(AppConfig::new(RawAppConfig::default()).expect("配置构造失败"), pool))` with a **`connect_lazy` MySQL pool pointed at an unreachable address**; requests dispatched in-process via `tower::ServiceExt::oneshot` (no TCP server). No config file/env reads; tests exercising DB paths assert the 500 failure path. Helpers `lazy_state/send/get_json/post_json` live in this file.
- **Backend unit tests**: synchronous, inline — `src/modules/role/models.rs` (permission code consistency, `perms_match` wildcards), `src/modules/user/models.rs` (Debug must not print the password hash), `src/config/mod.rs` (Debug must not print the DB URL or JWT secret) and `src/cli/command.rs` (Debug must not print the CLI password). Password hashing/verification moved into the library, so `util/password.rs` no longer exists.
- **Admin frontend**: no unit tests — `pnpm check` (lint + prettier) + `pnpm type-check` + `pnpm --filter admin build` are the gates; behavior verified manually in the browser (login → dashboard → users CRUD → profile → logout).
- **Uniapp**: no unit tests — `pnpm lint` + `pnpm type-check` + `pnpm build` (H5) are the gates; behavior verified manually (register → login → workbench → profile → logout).
- **Coverage**: no coverage config. CI (`ci.yml`) enforces fmt/clippy/test + spec/SDK drift on every push; no coverage gate.
- **Before touching auth/RBAC**: verify against `Perm` codes, `perms_match` wildcards, `DEFAULT_ROLE_PERMISSIONS`, and the two frontend permission helpers; breakage there is silent at compile time (JSON column + runtime checks).
