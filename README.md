# ai-guard

Local AI API relay with traffic logging, account routing, and async security auditing.

## What It Does

ai-guard sits between your AI clients and upstream providers (OpenAI-compatible, Anthropic-compatible), providing:

- **API Proxy** — Relay requests to upstream providers. Clients point to ai-guard instead of the real endpoint; it forwards and returns responses transparently.
- **Account Routing** — Configure multiple API keys/accounts per provider. ai-guard picks the best available one based on API type, model support, priority, and health. Failed accounts are automatically disabled with exponential backoff.
- **Traffic Logging** — Every request/response is logged with method, path, model, account, latency, and status. Sensitive fields (API keys, tokens, passwords) are automatically redacted before storage.
- **Security Auditing** — After each relay, an async task sends the redacted preview to an LLM (via OpenRouter) for risk assessment. Results appear as audit reports with risk level, title, and findings.

A web dashboard is embedded in the binary for managing accounts, browsing logs, and reviewing audit reports.

## Quick Start

```bash
# Build (includes frontend)
cargo build --release

# Run
./target/release/ai-guard
```

Open `http://127.0.0.1:8787` in your browser to access the dashboard.

Point your AI clients to `http://127.0.0.1:8787` instead of the upstream API:

```bash
# OpenAI-compatible
export OPENAI_BASE_URL=http://127.0.0.1:8787/v1

# Anthropic
export ANTHROPIC_BASE_URL=http://127.0.0.1:8787/anthropic
```

## Configuration

All settings can be set via CLI flags, environment variables, or a `.env` file in the current
working directory. Real environment variables take precedence over `.env` values.

| Flag | Env Variable | Default | Description |
|------|-------------|---------|-------------|
| `--bind` | `AI_GUARD_BIND` | `127.0.0.1:8787` | Listen address |
| `--data-dir` | `AI_GUARD_DATA_DIR` | `~/.ai-guard` | Data directory for state |
| `--openrouter-key` | `AI_GUARD_OPENROUTER_KEY` | — | OpenRouter API key (enables auditing) |
| `--openrouter-base-url` | `AI_GUARD_OPENROUTER_BASE_URL` | `https://openrouter.ai/api/v1` | OpenRouter base URL |
| `--audit-model` | `AI_GUARD_AUDIT_MODEL` | `qwen/qwen3-4b:free` | Model used for auditing |
| `--max-body-bytes` | `AI_GUARD_MAX_BODY_BYTES` | `4194304` | Max request body size (4 MB) |
| `--preview-bytes` | `AI_GUARD_PREVIEW_BYTES` | `4096` | Max preview size for logging/audit |
| `--resend-key` | `AI_GUARD_RESEND_KEY` | — | Resend API key for audit alert emails |
| `--resend-email` | `AI_GUARD_RESEND_EMAIL` | — | Recipient email for high/critical audit alerts; comma-separated values are supported |
| `--resend-from` | `AI_GUARD_RESEND_FROM` | `AI Guard <onboarding@resend.dev>` | Sender address for Resend alert emails |

## Proxy Routes

| Route | Upstream |
|-------|----------|
| `/v1/*` | OpenAI-compatible |
| `/openai/*` | OpenAI-compatible (prefix variant) |
| `/anthropic/*` | Anthropic-compatible |

## Management API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/config` | GET | Show runtime config |
| `/api/stats` | GET | Account/log/report counts |
| `/api/accounts` | GET | List accounts |
| `/api/accounts` | POST | Create account |
| `/api/accounts/{id}` | PUT | Update account |
| `/api/accounts/{id}` | DELETE | Delete account |
| `/api/accounts/{id}/check` | POST | Validate account connectivity |
| `/api/logs` | GET | List logs (with filters) |
| `/api/reports` | GET | List audit reports (with filters) |

## Development

```bash
# Install frontend dependencies
pnpm --dir frontend install

# Start backend (auto-reload)
cargo watch -x run

# Start frontend dev server (in another terminal)
pnpm --dir frontend run dev
```

The frontend dev server runs on `http://127.0.0.1:5173` and proxies API/proxy requests to the backend.

### Build Flags

| Env Variable | Description |
|-------------|-------------|
| `AI_GUARD_BUILD_FRONTEND=1` | Build frontend even in debug mode |
| `AI_GUARD_SKIP_FRONTEND_BUILD=1` | Skip frontend build in build.rs |
| `AI_GUARD_FRONTEND_REQUIRED=1` | Fail if frontend build fails |

## How It Works

1. Client sends a request to ai-guard (e.g. `POST /v1/chat/completions`)
2. ai-guard extracts the `model` field and selects an available upstream account
3. The request is forwarded to the upstream provider with proper auth headers
4. On success: response is returned to the client, request/response is logged (redacted), and an async audit is spawned
5. On failure: the account is marked as failed with backoff, and the error is returned to the client

Account selection prioritizes: enabled → matching API type → model supported → not in cooldown → highest priority → fewest failures → most recently updated.
