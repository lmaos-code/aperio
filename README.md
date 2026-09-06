# Aperio

A self-hosted MCP server that transforms an Obsidian vault into a persistent, remotely accessible knowledge base for AI agents.

Aperio syncs with Obsidian's own cloud service (preserving end-to-end encryption), then exposes your notes over the [Model Context Protocol](https://modelcontextprotocol.io/) so agents can read, search, and write notes programmatically.

## Features

- **MCP-native** — proper Streamable HTTP transport, not a REST wrapper
- **Sync-native** — uses Obsidian's sync protocol; this is your real vault, not a copy
- **OIDC auth** — works with Keycloak, PocketID, Authentik, or any OIDC provider
- **Read-write** — full CRUD via MCP tools (list, read, write, delete, search)
- **Observable** — live status page showing sync health, vault stats, and uptime
- **Helm-ready** — Kubernetes deployment with Gateway API HTTPRoute support

### MCP Tools

| Tool | Description |
|---|---|
| `list_notes` | List notes, optionally filtered by folder |
| `read_note` | Read a note's content and frontmatter |
| `search_notes` | Full-text search across all notes |
| `list_folders` | List vault folders with note counts |
| `write_note` | Create or overwrite a note |
| `delete_note` | Soft-delete (moves to `.trash/`) |
| `get_note_metadata` | Read frontmatter and tags only |

## Quick Start

### Docker

```bash
docker run -d \
  -p 3000:3000 \
  -v /path/to/vault:/vault \
  -e OIDC_ISSUER_URL=https://keycloak.example.com/realms/aperio/.well-known/openid-configuration \
  ghcr.io/lmaos-code/aperio:v1
```

### Helm

```bash
helm install aperio oci://ghcr.io/lmaos-code/charts/aperio \
  --set auth.issuerUrl=https://keycloak.example.com/realms/aperio
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `MCP_PORT` | `3000` | Server listen port |
| `OBSIDIAN_VAULT_DIR` | `/obsidian-vault` | Path to the vault |
| `RUST_LOG` | — | Log level (`info`, `debug`, `trace`) |
| `APERIO_VERSION` | `development` | Version string (set automatically in Docker) |
| `OIDC_ISSUER_URL` | — | OIDC discovery URL (**required in production**) |
| `LOCAL_AUTH` | — | Set `true` to disable auth (dev only) |
| `PUBLIC_URL` | — | Public base URL for OAuth metadata (derived from `Host` header if unset) |
| `OIDC_AUDIENCE` | — | Required `aud` claim |
| `OIDC_ISSUER` | — | Required `iss` claim |
| `OIDC_REQUIRED_CLAIMS` | — | Comma-separated required claims (e.g. `sub,email`) |
| `MCP_REALM` | `mcp` | Realm in `WWW-Authenticate` header |
| `MCP_SCOPES` | `mcp:read,mcp:write` | Required OAuth scopes |
| `SYNC_CHECK` | `none` | Sync health method: `file`, `heartbeat`, `kubernetes`, or `none` |

### Helm Values

```yaml
replicaCount: 1

auth:
  issuerUrl: https://keycloak.example.com/realms/aperio
  audience: ""
  requiredIssuer: ""
  requiredClaims: ""

httpRoute:
  enabled: true
  gatewayName: my-gateway
  hostnames:
    - aperio.example.com

persistence:
  size: 10Gi

sync:
  serverUrl: https://api.obsidian.md
  token: ""
  deviceName: aperio-k8s
```


## Authentication

Aperio requires OIDC authentication in production. It supports:

- **Authorization Code + PKCE** — for interactive sessions (Claude, browser-based agents)
- **Client Credentials** — for machine-to-machine automation

The server publishes [RFC 9728](https://datatracker.ietf.org/doc/rfc9728/) Protected Resource Metadata at `/.well-known/oauth-protected-resource`, enabling MCP clients to auto-discover the auth server.

## Endpoints

| Path | Description |
|---|---|
| `/mcp` | MCP Streamable HTTP endpoint |
| `/status` | Live status page (auto-refreshes via HTMX) |
| `/healthz` | Liveness/readiness probe |
| `/.well-known/oauth-protected-resource` | RFC 9728 metadata (public) |

## Roadmap - Soon™
- [ ] Semantic Search for your Agent trough coupled RAG-System with Aperio


## Development

### Prerequisites

- Rust 1.97+
- Docker & Docker Compose
- Helm (for chart linting)

### Local Setup

```bash
# Start Keycloak + Postgres
docker compose -f dev/compose.yml up -d

# Run Aperio
cargo run
```

The dev environment includes:
- **Keycloak** on `http://localhost:8081` (admin/admin)
- **Pre-configured realm** with test clients and users (`dev`/`password`)
- **Mock vault** at `dev/vault/`

### With Obsidian Sync

```bash
cp dev/sync.env .env.sync  # add your token
docker compose -f dev/compose.yml --profile sync up -d
```

### Useful Commands

```bash
bacon                          # just use bacon
cargo clippy -- -D warnings    # lint
cargo fmt --check              # format check
cargo test                     # run tests
helm lint deploy/charts/aperio # lint Helm chart
```
