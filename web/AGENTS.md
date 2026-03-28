# WEB FRONTEND

## OVERVIEW
SvelteKit admin UI package. Client-only app that talks to the Rust backend over `/api` and `/api/ws`.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| App shell | `src/routes/+layout.svelte` | Shared admin shell |
| Runtime mode | `src/routes/+layout.ts` | `ssr = false`, `prerender = false` |
| Route pages | `src/routes/` | Dashboard, logs, settings, videos, subscriptions |
| HTTP boundary | `src/lib/api.ts` | All fetch calls and auth token storage |
| WS boundary | `src/lib/ws.ts` | Reconnects, subscriptions, WS error toasts |
| Shared contracts | `src/lib/types.ts` | Backend response/request types |
| Shared stores | `src/lib/stores/` | Breadcrumb + filter state |
| App components | `src/lib/components/` | App-specific UI |
| UI wrappers | `src/lib/components/ui/` | Barrel-based reusable wrapper layer |

## CONVENTIONS
- Use `$lib/...` imports. Do not introduce `@/*`; `svelte.config.js` contains a placeholder alias, but repo code uses `$lib` consistently.
- All backend access goes through `src/lib/api.ts` and `src/lib/ws.ts`; avoid route-local fetch/WebSocket wrappers.
- Auth token is stored in localStorage by `api.ts`; changes to auth flow must respect both HTTP and WS paths.
- Dev server proxies `/api` and `/api/ws` to `localhost:12345` via `vite.config.ts`.
- Formatting is tabs, single quotes, no trailing commas, print width 100.
- ESLint allows unused variables only when prefixed with `_`; deprecated APIs are errors.

## TESTING / VALIDATION
- No dedicated frontend test suite is present.
- Expected safety checks are `bun run lint`, `bun run check`, and `bun run build`.

## ANTI-PATTERNS
- Do not bypass `src/lib/api.ts` for REST calls.
- Do not open raw `WebSocket` connections from pages when `src/lib/ws.ts` already models subscriptions and reconnect behavior.
- Do not add SSR/prerender assumptions; this package is intentionally client-only.
- Do not mix generic UI wrappers with app-specific business logic; keep that split between `components/ui` and the rest of `components`.

## NOTES
- `components.json` and current imports indicate shadcn-svelte style aliases centered on `$lib/components/ui`.
- If backend payloads change, update `src/lib/types.ts` and then adjust affected routes.
