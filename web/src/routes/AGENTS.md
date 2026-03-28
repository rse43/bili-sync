# ROUTE AUTHORING

## OVERVIEW
Client-side admin pages. Routes own page-level data loading, breadcrumb setup, toast messaging, and subscription cleanup.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Dashboard | `+page.svelte` | Metrics, task trigger, sysinfo/task subscriptions |
| Logs | `logs/+page.svelte` | Live log stream via WS |
| Settings | `settings/+page.svelte` | Auth/config/notifier workflows |
| Video list | `videos/+page.svelte` | Filters, batch actions, status updates |
| Video detail | `video/[id]/+page.svelte` | Single-video actions |
| Video sources | `video-sources/+page.svelte` | Largest CRUD-style route |
| Quick subscriptions | `me/*/+page.svelte` | Favorites, collections, uppers |

## CONVENTIONS
- Set the breadcrumb early with `setBreadcrumb(...)` from `$lib/stores/breadcrumb`.
- Use `toast.success/error/info/warning` for user-visible outcomes; this is the standard error/success channel across routes.
- Call the shared API facade from `$lib/api`; do not duplicate request plumbing per page.
- Use shared stores/components before inventing route-local state machines.
- If a page subscribes to logs/tasks/sysinfo, clean up the unsubscribe callback in `onMount` teardown.
- Route labels and user-facing text are Chinese.

## ANTI-PATTERNS
- Do not fetch directly with `fetch(...)` from pages when the same operation belongs in `$lib/api`.
- Do not leave WS subscriptions hanging after unmount.
- Do not hide backend failures in console-only logging; surface them with toasts.
- Do not introduce inconsistent navigation labels; sidebar grouping in `$lib/components/app-sidebar.svelte` is the source of truth.

## NOTES
- `video-sources/+page.svelte` and `+page.svelte` are large and representative; follow their page ownership pattern instead of introducing extra service layers inside `routes/`.
- `me/` pages are a small cohesive family and should stay aligned in UX and API style.
