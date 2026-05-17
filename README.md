# anide.app

One app. Every tool a developer needs daily.

Stop juggling Postman, DBeaver, Redis Insight, MinIO Browser, GitHub Desktop, and Docker Desktop. Anide puts all of them in a single native desktop app — built on Tauri/Rust, not Electron. Everything is stored in a `.anide/` folder at your repo root, versioned alongside your code. No accounts. No cloud sync. Credentials stay on your machine.

---

## Why

- **Too many apps eating RAM** — six tools open means six Electron processes running in the background
- **Context switching kills flow** — alt-tabbing between tools breaks concentration constantly
- **Sharing configs is painful** — every teammate has their own Postman setup, nothing is in sync
- **Credentials leaving your machine** — cloud-synced workspaces mean your API keys and DB passwords live on someone else's server
- **Nothing talks to each other** — every tool is a silo; your API client has no idea what's in your `.env`

---

## Roadmap

### REST Client

Replaces: Postman, Insomnia, Bruno

- [ ] REST, GraphQL, gRPC, WebSocket, SSE requests
- [ ] Auth: Bearer/JWT, API Key, Basic, OAuth 2.0, mTLS
- [ ] Environment variables pulled from your `.env` files automatically
- [ ] Collections committed to Git alongside your code
- [ ] Import from Postman JSON, OpenAPI, curl

### Database

Replaces: DBeaver, TablePlus, MongoDB Compass

- [ ] Connect to PostgreSQL, MySQL/MariaDB, SQLite, MongoDB
- [ ] Write and run queries with results table
- [ ] Explore schema and table structure
- [ ] Edit rows directly
- [ ] Track and compare migrations across environments

### KV / Cache

Replaces: Redis Insight, Another Redis Desktop Manager

- [ ] Connect to Redis, Valkey, KeyDB, Dragonfly
- [ ] Browse and inspect keys with TTL visualization
- [ ] Debug pub/sub channels
- [ ] Run CLI commands inline
- [ ] Flush keys during dev without leaving the app

### Object Storage

Replaces: MinIO Browser, Cyberduck, AWS S3 Console

- [ ] Connect to AWS S3, Cloudflare R2, MinIO, Backblaze B2, DigitalOcean Spaces
- [ ] Browse, upload, and download files
- [ ] Preview images and files inline
- [ ] Generate presigned URLs
- [ ] Manage bucket policies

### Git

Replaces: GitHub Desktop, GitKraken, Fork

- [x] Stage files and commit
- [x] View diffs before committing
- [x] Create, switch, and manage branches
- [x] Push, pull, fetch
- [x] View commit history
- [ ] Resolve merge conflicts

### Docker

Replaces: Docker Desktop, Lazydocker

- [x] Start, stop, restart containers
- [x] Stream container logs live
- [x] `docker-compose up/down`
- [ ] Browse images and volumes
- [ ] Check port mappings
- [x] Exec into a running container

---

## Stack

- **Frontend** — Svelte 5, SvelteKit 2, Tailwind CSS 4, shadcn-svelte
- **Backend** — Rust, Tauri 2
- **Editor** — CodeMirror 6, Tiptap 3
- **Package manager** — Bun

---

## Dev

```bash
bun install
bun run tauri dev
```

Build:

```bash
bun run tauri build
```

---

## How it works

Opening a project creates a `.anide/` folder at the repo root. All your requests, notes, and config live there as plain files — readable, diffable, and committed to Git with the rest of your code.

```
your-project/
├── .anide/
│   ├── README.md          ← project notes, editable in-app
│   └── api/               ← saved requests (coming soon)
├── .env
├── .env.local
└── ...
```

Pull requests get richer. Onboarding gets faster. Your API keys never leave your machine.

---

> [anide.app](https://anide.app) · Built with Svelte + Tauri