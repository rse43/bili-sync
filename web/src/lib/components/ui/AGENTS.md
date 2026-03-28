# UI WRAPPER LAYER

## OVERVIEW
Reusable UI wrapper components. This directory is the generic presentation layer, not the place for app-specific data or business rules.

## STRUCTURE
```text
ui/
├── <component>/
│   ├── index.ts
│   └── *.svelte
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Registry/alignment | `web/components.json` | `$lib/components/ui` alias source |
| Shared styling helpers | `web/src/lib/utils.ts` | `cn(...)` helper |
| Route usage examples | `web/src/routes/` | Real import/usage patterns |

## CONVENTIONS
- Preserve the barrel pattern: each component folder exports through `index.ts`.
- Keep imports aligned with repo usage: pages/components import from `$lib/components/ui/<name>/index.js`.
- Match existing wrapper style: thin composition around generic UI primitives plus project styling.
- Keep this layer presentation-focused; route logic, API calls, and app-specific domain behavior belong elsewhere.
- Follow the formatter/plugins already configured in `web/.prettierrc`; Tailwind class ordering is automatic.

## ANTI-PATTERNS
- Do not add backend calls, stores, or route-specific side effects here.
- Do not bypass the established folder-per-component export shape.
- Do not mix app-specific “custom” widgets into this generic wrapper layer; use sibling `components/` or `components/custom/` instead.

## NOTES
- This subtree is large enough to deserve its own rules because it behaves like an internal component library.
- If a wrapper starts needing business-language props or API data knowledge, it probably belongs outside `ui/`.
