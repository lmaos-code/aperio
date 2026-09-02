# Idea Manifesto: Aperio

## What

A self-hosted MCP (Model Context Protocol) server that transforms an Obsidian vault into a persistent, remotely accessible knowledge base for AI agents.

## How

A single container runs two processes managed by a process supervisor:

- **Obsidian Headless Sync** keeps the vault synchronized with Obsidian's cloud service, preserving end-to-end encryption and conflict resolution.
- **Rust MCP Server** reads the synchronized vault from the local filesystem, exposes it over the MCP protocol, and gates access behind OIDC authentication.

Both processes share a PersistentVolumeClaim mounted at `/vault/`. The container owns the entire lifecycle — no sidecar coordination, no inter-container communication.

## Principles

1. **Self-hosted.** Your vault, your server, your data. No third-party dependency.
2. **Sync-native.** Uses Obsidian's own sync protocol — not a copy, not a mirror, the real vault.
3. **OIDC-compatible.** Works with any standards-compliant identity provider (PocketID, Authentik, Keycloak, etc.).
4. **MCP-native.** Implements the Model Context Protocol properly — not a REST wrapper, not a filesystem browser.
5. **Read-write.** Full CRUD operations on notes, not just read access.

## Capabilities (MVP)

| Tool | Description |
|------|-------------|
| `list_notes` | List all notes, optionally filtered by folder |
| `read_note` | Read a note's content and frontmatter |
| `write_note` | Create or overwrite a note |
| `delete_note` | Soft-delete a note to .trash/ |
| `search_notes` | Full-text search across all notes |
| `list_folders` | List vault folders with note counts |
| `get_note_metadata` | Read frontmatter and tags without body |

## Authentication Flows

- **Authorization Code + PKCE** — for interactive user sessions (Claude, browser-based agents)
- **Client Credentials** — for machine-to-machine automation (scheduled agents, CI/CD pipelines)

## Deployment

Runs as a single Kubernetes pod with one container and a PersistentVolumeClaim:

```
┌─────────────────────────────────────────────┐
│  Pod                                        │
│  ┌──────────────────────────────────────┐   │
│  │ aperio (Rust)                        │   │
│  │  ├─ sync ── ob sync --cont ──▶┐      │   │
│  │  └─ mcp ──────────────────▶┐  │      │   │
│  │       │                    │  │      │   │
│  │       ▼                    ▼  │      │   │
│  │    /vault/  ◀──────────────┘  │      │   │
│  └──────────────────────────────────────┘   │
│         │                                   │
│         ▼                                   │
│   PersistentVolumeClaim (vault storage)     │
└─────────────────────────────────────────────┘
```

The sync process runs as a background task managed by the main Rust binary. A process supervisor (e.g. `tokio::spawn` or `nix::unistd::fork`) handles restarts and graceful shutdown. The PVC is required — the container filesystem is ephemeral and a full re-sync from Obsidian on every restart is slow and unnecessary.

## Status Page

A lightweight status page served by the Rust MCP server on a dedicated HTTP port (default `:8080`).

- **`/status`** — Human-readable status page (htmx). Shows sync state, last successful sync, vault stats (note count, folder count), server uptime.
- **`/healthz`** — Reserved for Kubernetes liveness/readiness probes. Returns `200 OK` when the process is alive. No business logic.

The status page is read-only, unauthenticated, and intentionally minimal. It exists so the admin can glance at a dashboard and confirm the service is healthy without poking at logs.

## Non-Goals

- Replacing Obsidian's UI or desktop app
- Providing a web frontend for note editing (use Obsidian for that)
- Implementing custom sync protocols (use Obsidian's)
- Multi-user access control per note (MVP is single-user)

## References

- [MCP Specification](https://modelcontextprotocol.io)
- [Obsidian Headless Sync](https://obsidian.md/help/sync/headless)
- [RFC 9728 - OAuth Protected Resource Metadata](https://datatracker.ietf.org/doc/html/rfc9728)
