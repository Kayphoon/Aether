# FRONTEND KNOWLEDGE

## OVERVIEW

`frontend` is a Vue 3 + Vite management console and public guide app. It uses Pinia, Vue Router, Axios, Vitest, Tailwind, Radix Vue, lucide-vue-next, Chart.js, and local design variables in `src/style.css`.

## STRUCTURE

```text
frontend/
|-- package.json          # scripts and dependencies
|-- vite.config.ts        # Vite config, proxy, alias, manual chunks, version injection
|-- vitest.config.ts      # jsdom test setup and coverage config
|-- src/main.ts           # Vue bootstrap
|-- src/App.vue           # app shell, auth sync, global UI containers
|-- src/router/           # route map and guards
|-- src/api/              # API modules and central Axios client
|-- src/stores/           # Pinia stores
|-- src/features/         # business-domain components and tests
|-- src/views/            # public, user, shared, and admin pages
|-- src/components/       # shared layout, chart, icon, and UI components
`-- src/tests/            # Vitest setup
```

## WHERE TO LOOK

| Task | Location | Notes |
|---|---|---|
| App bootstrap | `src/main.ts` | Creates Vue app, Pinia, router, global styles |
| Route behavior | `src/router/index.ts`, `src/router/guards` | Public, dashboard, admin routes plus auth/admin/module gates |
| API requests | `src/api/client.ts` | Axios instance, token injection, 401 refresh, account-level 403 handling |
| Auth state | `src/stores/auth.ts` | Login, logout, user loading, permissions |
| Feature UI | `src/features/<domain>` | Domain components, helpers, and colocated tests |
| Pages | `src/views/{admin,public,shared,user}` | Route-level screens |
| Shared UI | `src/components` | Reusable components before adding new page-local UI |
| Tests | `src/**/__tests__/*.spec.ts`, `src/tests/vitest.setup.ts` | Vitest with jsdom and storage mocks |

## CONVENTIONS

Use the `@` alias for `frontend/src` imports. Keep request behavior through `src/api/client.ts` so auth headers, demo mocks, refresh coordination, and cross-tab sync remain centralized.

Routes use metadata for auth, admin, and module access. Match existing guard patterns instead of adding page-local redirect logic.

Feature code is domain-organized. Prefer `src/features/<domain>` or an existing `src/views/<area>` branch over adding broad utility files.

The Vite dev server proxies `/api/`, `/v1/`, `/health`, and `/_gateway/` to the backend `APP_PORT`, defaulting to `8084`.

## ANTI-PATTERNS

Do not bypass `ApiClient` for authenticated backend calls.

Do not add new global CSS variables or theme behavior without checking `src/style.css`.

Do not treat `npm run lint` as read-only; the script runs `eslint . --fix`.

## COMMANDS

```bash
cd frontend && npm run dev
cd frontend && npm run build
cd frontend && npm run build:with-typecheck
cd frontend && npm run test:run
cd frontend && npm run type-check
cd frontend && npm run lint
```
