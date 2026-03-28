# BILI_SYNC BACKEND

## OVERVIEW
Main Rust binary crate. Owns runtime startup, Axum API, scheduler/downloader tasks, config persistence, and frontend embedding.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Runtime start | `src/main.rs` | Initializes logger, DB, config, FFmpeg check, task spawning |
| HTTP server | `src/task/http_server.rs` | Serves API + embedded frontend |
| Download scheduler | `src/task/` | Long-running task execution |
| API surface | `src/api/routes/` | Route modules merged under `/api` |
| Auth middleware | `src/api/routes/mod.rs` | Header + WebSocket protocol auth |
| Config model/rules | `src/config/current.rs` | Validation and defaults |
| DB bootstrap | `src/database.rs` | Migration gate, connection tuning |
| External Bilibili logic | `src/bilibili/` | API integrations and parsing |
| Shared helpers | `src/utils/` | filenames, NFO, rules, logging, signals |

## CONVENTIONS
- `main.rs` starts exactly two tracked long-lived tasks: HTTP service and timed downloader.
- API modules are aggregated through `src/api/routes/mod.rs`; add routes there instead of scattering ad hoc routers.
- UI-facing auth supports both `Authorization` header and base64url token via `Sec-WebSocket-Protocol` for WS.
- Config is loaded/saved through the database, not a mutable on-disk config file.
- Validation errors are accumulated into a single multi-line failure in `Config::check()`.
- SQLite uses WAL, busy timeout, and separate migration connection behavior to avoid migration-order issues.
- Frontend assets are expected to be prebuilt and embedded; backend changes can indirectly depend on `web/build` shape.

## TESTING
- Keep tests inline with implementation using `#[cfg(test)] mod tests`.
- Async tests use `#[tokio::test]`.
- Manual real-service tests are marked `#[ignore = "only for manual test"]`.
- Some manual tests initialize `VersionedConfig::init_for_test(...)` against `./test.sqlite`; do not convert those into always-on unit tests without isolating external dependencies.

## ANTI-PATTERNS
- Do not allow direct upgrades from versions older than 2.6.x; `src/database.rs` intentionally rejects them.
- Do not bypass `Config::check()` semantics with looser validation.
- Do not split API access patterns away from `src/api/routes/` aggregation unless the routing model itself changes.
- Do not assume one task failing should be ignored; current runtime cancels peers on abnormal task exit.

## NOTES
- This crate is the only releasable Cargo package in the workspace.
- If a change touches request/response shape, also inspect `web/src/lib/types.ts` and `web/src/lib/api.ts`.
