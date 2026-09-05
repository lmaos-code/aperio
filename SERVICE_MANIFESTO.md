# Idea Manifesto: Aperio

## What

A self-hosted MCP (Model Context Protocol) server that transforms an Obsidian vault into a persistent, remotely accessible knowledge base for AI agents.

## How

A Kubernetes pod runs two containers that share a PersistentVolumeClaim:

- **Obsidian Headless Sync** container keeps the vault synchronized with Obsidian's cloud service, preserving end-to-end encryption and conflict resolution.
- **Aperio** container runs the Rust MCP server, reads the synchronized vault from the shared volume, exposes it over the MCP protocol, and gates access behind OIDC authentication.

The containers share storage via a PVC mounted at `/vault/`. Kubernetes manages container lifecycle, restarts, and health checks. Aperio monitors sync health via file-based heartbeats on the shared volume.

## Principles

1. **Self-hosted.** Your vault, your server, your data. No third-party dependency.
2. **Sync-native.** Uses Obsidian's own sync protocol — not a copy, not a mirror, the real vault.
3. **OIDC-compatible.** Works with any standards-compliant identity provider (PocketID, Authentik, Keycloak, etc.).
4. **MCP-native.** Implements the Model Context Protocol properly — not a REST wrapper, not a filesystem browser.
5. **Read-write.** Full CRUD operations on notes, not just read access.
6. **Observable.** Status page shows sync health, vault stats, and server status at a glance.

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

Runs as a Kubernetes pod with two containers and a PersistentVolumeClaim:

```
┌─────────────────────────────────────────────────────────────────┐
│  Pod                                                            │
│  ┌───────────────────────────┐  ┌───────────────────────────┐   │
│  │ aperio                    │  │ sync                      │   │
│  │  └─ mcp ──────────────┐   │  │  └─ ob sync --cont ──┐   │   │
│  │                       │   │  │                      │   │   │
│  │       ┌───────────────┘   │  │       ┌──────────────┘   │   │
│  │       ▼                   │  │       ▼                  │   │
│  │    /vault/ ◀──────────────┴──┴──── /vault/              │   │
│  └───────────────────────────┘  └───────────────────────────┘   │
│                         │                                       │
│                         ▼                                       │
│           PersistentVolumeClaim (vault storage)                 │
└─────────────────────────────────────────────────────────────────┘
```

Each container runs independently with its own lifecycle. Kubernetes manages restarts and health checks. The PVC is required — the container filesystem is ephemeral and a full re-sync from Obsidian on every restart is slow and unnecessary.

**Sync monitoring:** Aperio monitors sync health by checking file modification times on the shared volume. If vault files haven't been modified recently, sync is flagged as stale on the status page.

## Status Page

A lightweight status page served by the Rust MCP server (default `:3000/status`).

- **`/status`** — Human-readable status page (askama + htmx). Shows sync state (file-based heartbeat), vault stats (note count, folder count), server uptime, auth status.
- **`/status/stats`** — HTMX partial endpoint for auto-refreshing stats every 5 seconds.
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
