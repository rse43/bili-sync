# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-28
**Commit:** 8a58199
**Branch:** main

## OVERVIEW
Rust-first monorepo for bili-sync: a Tokio/Axum binary, a separate SvelteKit admin UI, and a VitePress docs site. Production serving is backend-first; the Rust app embeds the built frontend.

## STRUCTURE
```text
bili-sync/
├── crates/                 # Rust workspace members
│   ├── bili_sync/          # main binary: API, downloader, config, runtime tasks
│   ├── bili_sync_entity/   # SeaORM entities and DB custom types
│   └── bili_sync_migration/# schema migrations
├── web/                    # SvelteKit admin UI, built separately with Bun
├── docs/                   # VitePress docs site, Simplified Chinese
├── scripts/                # one-off/support scripts, not main product code
├── .github/workflows/      # CI, release, docs deploy
├── Justfile                # local task runner
├── Cargo.toml              # workspace, release metadata, shared deps
└── Dockerfile              # packages prebuilt release tarballs
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App startup | `crates/bili_sync/src/main.rs` | Spawns HTTP server + download task |
| API routing | `crates/bili_sync/src/api/routes/` | All `/api` route modules live here |
| Config rules | `crates/bili_sync/src/config/current.rs` | Runtime validation and defaults |
| DB setup/migration | `crates/bili_sync/src/database.rs` | Upgrade gating and SQLite options |
| Frontend API boundary | `web/src/lib/api.ts` | HTTP entry point for UI |
| Frontend WebSocket boundary | `web/src/lib/ws.ts` | Shared subscriptions/reconnect logic |
| Frontend pages | `web/src/routes/` | Client-only admin routes |
| Shared UI wrappers | `web/src/lib/components/ui/` | shadcn-svelte style wrapper layer |
| Docs site config | `docs/.vitepress/config.mts` | Nav, sidebar, search, version text |
| Local build flow | `Justfile` | Frontend build precedes Rust build |
| CI/release | `.github/workflows/` | Validation, binary release, docs deploy |

## CONVENTIONS
- Rust workspace uses `crates/*`; default member is `crates/bili_sync`.
- Release version is centralized in root `Cargo.toml`; release automation also rewrites `docs/.vitepress/config.mts`, `docs/introduction.md`, and `web/package.json`.
- Frontend/docs use Bun in CI; do not assume npm.
- Frontend is built before release binary packaging; local `just build*` targets do the same.
- Config is database-backed since v2.6.0; docs and UI both treat Web UI as the primary config surface.
- Rust formatting is non-default: grouped imports, module granularity, 120-column width.
- Frontend formatting is tabs + single quotes + no trailing commas; Prettier also organizes imports and sorts Tailwind classes.
- Frontend runtime is client-only (`ssr = false`, `prerender = false`).
- Rust tests are inline `#[cfg(test)] mod tests`; async tests use `#[tokio::test]`.

## ANTI-PATTERNS (THIS PROJECT)
- Do not build/release the Rust binary without first building `web/`; release pipeline depends on embedded frontend assets.
- Do not use `latest` Docker tags in docs/examples; repo docs explicitly recommend pinning versions.
- Do not upgrade directly from pre-2.6.0 data to current; backend blocks this path in `database.rs`.
- Do not assume config lives in a standalone file; runtime config is stored in the database.
- Do not use bare `Vec` entity fields for cross-database array behavior; wrapper types are required.
- Do not introduce deprecated TS APIs in the frontend; ESLint treats them as errors.

## UNIQUE STYLES
- Chinese-first product/docs/log messages.
- Backend serves product runtime; frontend is an embedded asset bundle, not a separately deployed app.
- Docs are a separate package but mainly reflect root conventions rather than their own contributor policy.
- Some Rust tests are intentionally manual/ignored and may rely on `./test.sqlite` plus real external services.

## COMMANDS
```bash
# local
just build-frontend
just build
just build-debug
just debug

# backend checks
cargo +nightly fmt --check
cargo clippy -- -D warnings
cargo test

# frontend
cd web && bun install --frozen-lockfile
cd web && bun run lint
cd web && bun run check
cd web && bun run build

# docs
cd docs && bun install --frozen-lockfile
cd docs && bun run docs:build
```

## NOTES
- LSP code map was unavailable in this environment (`rust-analyzer` and frontend LSP not installed), so this file relies on direct repo evidence.
- `scripts/` is utility/one-off material; avoid treating it as the canonical implementation path.
- `docs/` is distinct tooling, but currently small enough that root guidance covers it.
