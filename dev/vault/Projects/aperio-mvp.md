---
title: Aperio MVP
tags:
  - project
  - mcp
  - rust
status: active
created: 2025-01-15
---

# Aperio MVP

## Overview

Build a self-hosted MCP server that exposes an Obsidian vault to AI agents.

## Goals

1. Read notes from vault directory
2. Expose notes via MCP protocol
3. Support basic CRUD operations
4. OIDC authentication

## Architecture

```
┌─────────────────────────────────┐
│  Aperio (Rust)                  │
│  ├─ MCP Server (axum)          │
│  ├─ Vault Reader (tokio::fs)   │
│  └─ Auth Middleware (JWT)      │
│         │                      │
│         ▼                      │
│    /vault/ (PVC)               │
└─────────────────────────────────┘
```

## Progress

- [x] Project setup
- [x] Config loading
- [ ] Vault reading
- [ ] MCP tools
- [ ] Auth

## Links

- [[index|Home]]
- [[Reference/glossary|Glossary]]
