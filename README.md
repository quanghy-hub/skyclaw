# SkyClaw

Cloud-native Rust AI agent runtime. Interact with your agent entirely through messaging apps -- no SSH, no config files, no setup complexity.

## Overview

SkyClaw is an autonomous AI agent runtime written in 100% Rust. Users control their agent through Telegram, Discord, Slack, WhatsApp, or a local CLI. Credentials, files, and commands are sent as naturally as chatting. The runtime handles context assembly, AI model calls, tool execution, and response streaming -- all in a single static binary under 10 MB.

SkyClaw runs in two modes from the same binary:

- **Cloud mode** -- headless daemon on a VPS or container, TLS-enabled, binds to all interfaces
- **Local mode** -- runs on your laptop, localhost only, optional GUI with a headed browser

### Key Differentiators

- **Messaging-first UX**: messaging apps *are* the control plane -- send API keys, .env files, and commands via chat
- **Bi-directional file transfer**: send and receive files through any channel, with presigned URLs for large objects
- **Deny-by-default security**: mandatory sandboxing, allowlist-only channels, encrypted vault, audit logging
- **Trait-based extensibility**: 12 core traits define every subsystem; add a channel, provider, or tool by implementing a trait
- **Ecosystem compatibility**: reads ZeroClaw TOML configs and OpenClaw YAML configs out of the box

## Features

| Category | Details |
|----------|---------|
| **AI Providers** | Anthropic Claude, OpenAI-compatible (GPT-4, Ollama, vLLM), Google Gemini, Mistral, Groq |
| **Channels** | Telegram, Discord, Slack, WhatsApp, CLI REPL |
| **Memory** | SQLite (local), PostgreSQL (cloud), Markdown files (OpenClaw compat) -- hybrid vector + keyword search |
| **Tools** | Shell, file operations, browser automation, Git, HTTP requests, screenshots |
| **Security** | ChaCha20-Poly1305 encrypted vault, Ed25519 skill signing, workspace sandboxing |
| **Automation** | HEARTBEAT.md periodic checker, persistent cron scheduler |
| **Observability** | Structured JSON logging (tracing), OpenTelemetry export, `/health` endpoint |
| **File Transfer** | Inline attachments for small files, presigned URLs for large files, credential file parsing |
| **Skills** | SKILL.md with YAML frontmatter, OpenClaw/ZeroClaw format compatibility |

## Quick Start

### Prerequisites

- Rust 1.82+ (for building from source)
- Docker (for container deployment)
- An API key for at least one AI provider

### Option A: Docker (recommended)

```bash
docker pull ghcr.io/skyclaw/skyclaw:latest

docker run -d \
  --name skyclaw \
  -p 8080:8080 \
  -v ~/.skyclaw:/var/lib/skyclaw \
  -e ANTHROPIC_API_KEY=sk-ant-your-key \
  -e TELEGRAM_BOT_TOKEN=123456:ABC-DEF \
  ghcr.io/skyclaw/skyclaw:latest
```

### Option B: Pre-built Binary

Download the latest release for your platform from the [Releases](https://github.com/skyclaw/skyclaw/releases) page.

```bash
# Linux (x86_64)
curl -L https://github.com/skyclaw/skyclaw/releases/latest/download/skyclaw-x86_64-linux-musl -o skyclaw
chmod +x skyclaw

# Start the gateway
./skyclaw start
```

### Option C: Build from Source

```bash
git clone https://github.com/skyclaw/skyclaw.git
cd skyclaw
cargo build --release
./target/release/skyclaw start
```

### First Run

1. Copy and edit the default configuration:

```bash
mkdir -p ~/.skyclaw
cp config/default.toml ~/.skyclaw/config.toml
```

2. Set your AI provider:

```toml
[provider]
name = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-sonnet-4-6"
```

3. Enable a channel (e.g., Telegram):

```toml
[channel.telegram]
enabled = true
token = "${TELEGRAM_BOT_TOKEN}"
allowlist = ["your_telegram_username"]
file_transfer = true
```

4. Start SkyClaw:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export TELEGRAM_BOT_TOKEN="123456:ABC-..."
skyclaw start
```

5. Or use the local CLI for quick testing:

```bash
skyclaw chat
```

## CLI Reference

```
skyclaw start [--mode cloud|local|auto] [--gui]   Start the gateway daemon
skyclaw chat                                       Interactive CLI chat
skyclaw status                                     Show running state and health
skyclaw config validate                            Validate configuration
skyclaw config show                                Print resolved configuration
skyclaw skill list                                 List installed skills
skyclaw skill info <name>                          Show skill details
skyclaw skill install <path>                       Install a skill from a path
skyclaw migrate --from openclaw|zeroclaw <path>    Migrate from another runtime
skyclaw version                                    Show version info
```

Global flags:

| Flag | Description |
|------|-------------|
| `-c, --config <path>` | Path to config file |
| `--mode <mode>` | Runtime mode: `cloud`, `local`, or `auto` (default: `auto`) |

## Configuration Reference

SkyClaw uses TOML configuration. Sources are loaded in this order (later overrides earlier):

1. Compiled defaults
2. `/etc/skyclaw/config.toml`
3. `~/.skyclaw/config.toml`
4. `./config.toml` (workspace)
5. Environment variables (`SKYCLAW_*`)
6. CLI flags
7. `vault://` URIs resolved at runtime

### Full Configuration Table

| Section | Key | Type | Default | Description |
|---------|-----|------|---------|-------------|
| `[skyclaw]` | `mode` | `string` | `"auto"` | Runtime mode: `"cloud"`, `"local"`, or `"auto"` |
| | `tenant_isolation` | `bool` | `false` | Enable tenant isolation (future multi-tenant) |
| `[gateway]` | `host` | `string` | `"127.0.0.1"` | Bind address |
| | `port` | `u16` | `8080` | Bind port |
| | `tls` | `bool` | `false` | Enable TLS |
| | `tls_cert` | `string?` | `null` | Path to TLS certificate |
| | `tls_key` | `string?` | `null` | Path to TLS private key |
| `[provider]` | `name` | `string?` | `null` | Provider name: `"anthropic"`, `"openai-compatible"`, `"google"`, `"mistral"`, `"groq"` |
| | `api_key` | `string?` | `null` | API key (supports `${ENV_VAR}` and `vault://` URIs) |
| | `model` | `string?` | `null` | Model identifier |
| | `base_url` | `string?` | `null` | Custom API base URL (for OpenAI-compatible endpoints) |
| `[memory]` | `backend` | `string` | `"sqlite"` | Memory backend: `"sqlite"`, `"postgres"`, `"markdown"` |
| | `path` | `string?` | `null` | Database file path (SQLite) or memory directory (Markdown) |
| | `connection_string` | `string?` | `null` | PostgreSQL connection string |
| `[memory.search]` | `vector_weight` | `f32` | `0.7` | Weight for vector similarity in hybrid search |
| | `keyword_weight` | `f32` | `0.3` | Weight for keyword matching in hybrid search |
| `[vault]` | `backend` | `string` | `"local-chacha20"` | Vault backend |
| | `key_file` | `string?` | `null` | Path to vault encryption key |
| `[filestore]` | `backend` | `string` | `"local"` | File storage backend: `"local"` or `"s3"` |
| | `bucket` | `string?` | `null` | S3 bucket name |
| | `region` | `string?` | `null` | S3 region |
| | `path` | `string?` | `null` | Local filesystem path |
| `[security]` | `sandbox` | `string` | `"mandatory"` | Sandbox mode (always `"mandatory"`) |
| | `file_scanning` | `bool` | `true` | Scan uploaded files for secrets |
| | `skill_signing` | `string` | `"required"` | Require Ed25519 skill signatures |
| | `audit_log` | `bool` | `true` | Enable audit logging |
| | `rate_limit.requests_per_minute` | `u32?` | `null` | Rate limit per user |
| `[heartbeat]` | `interval` | `string` | `"30m"` | How often to check HEARTBEAT.md |
| | `checklist` | `string` | `"HEARTBEAT.md"` | Path to heartbeat file |
| `[cron]` | `storage` | `string` | `"sqlite"` | Cron job persistence backend |
| `[tools]` | `shell` | `bool` | `true` | Enable shell tool |
| | `browser` | `bool` | `true` | Enable browser automation tool |
| | `file` | `bool` | `true` | Enable file operations tool |
| | `git` | `bool` | `true` | Enable Git tool |
| | `cron` | `bool` | `true` | Enable cron tool |
| | `http` | `bool` | `true` | Enable HTTP request tool |
| `[tunnel]` | `provider` | `string` | -- | Tunnel provider: `"cloudflare"`, `"ngrok"`, etc. |
| | `token` | `string?` | `null` | Tunnel authentication token |
| | `command` | `string?` | `null` | Custom tunnel command |
| `[observability]` | `log_level` | `string` | `"info"` | Log level: `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"` |
| | `otel_enabled` | `bool` | `false` | Enable OpenTelemetry export |
| | `otel_endpoint` | `string?` | `null` | OpenTelemetry collector endpoint |
| `[channel.<name>]` | `enabled` | `bool` | `false` | Enable this channel |
| | `token` | `string?` | `null` | Bot/API token |
| | `allowlist` | `string[]` | `[]` | Allowed user IDs or usernames |
| | `file_transfer` | `bool` | `true` | Enable file transfer for this channel |
| | `max_file_size` | `string?` | `null` | Maximum file size override |

## Channel Setup Guides

### Telegram

1. Create a bot via [@BotFather](https://t.me/BotFather) on Telegram.
2. Copy the bot token.
3. Add the token to your config:

```toml
[channel.telegram]
enabled = true
token = "${TELEGRAM_BOT_TOKEN}"
allowlist = ["your_username"]
file_transfer = true
```

4. Set the environment variable and start SkyClaw:

```bash
export TELEGRAM_BOT_TOKEN="123456:ABC-DEF..."
skyclaw start
```

5. Open the bot in Telegram and send a message. Files up to 50 MB are transferred inline; larger files use presigned URLs.

### Discord

1. Create an application at the [Discord Developer Portal](https://discord.com/developers/applications).
2. Add a bot and copy the token.
3. Invite the bot to your server with the appropriate permissions (Send Messages, Attach Files, Read Message History).
4. Configure:

```toml
[channel.discord]
enabled = true
token = "${DISCORD_BOT_TOKEN}"
allowlist = ["your_discord_user_id"]
file_transfer = true
```

5. The bot handles DMs and configured guild channels. Files up to 25 MB (free) or 500 MB (Nitro) are inline; larger files use presigned URLs.

### Slack

1. Create a Slack App at [api.slack.com/apps](https://api.slack.com/apps).
2. Enable Socket Mode or Events API.
3. Add bot scopes: `chat:write`, `files:read`, `files:write`, `im:history`, `channels:history`.
4. Install the app to your workspace and copy the Bot User OAuth Token.
5. Configure:

```toml
[channel.slack]
enabled = true
token = "${SLACK_BOT_TOKEN}"
allowlist = ["U0123456789"]
file_transfer = true
```

6. Slack supports thread-aware conversations and files up to 1 GB inline.

### WhatsApp

1. Set up a [WhatsApp Business API](https://developers.facebook.com/docs/whatsapp/) account.
2. Obtain an API token from the Meta Developer Portal.
3. Configure:

```toml
[channel.whatsapp]
enabled = true
token = "${WHATSAPP_API_TOKEN}"
allowlist = ["+1234567890"]
file_transfer = true
```

4. The bot supports QR code pairing. Files up to 2 GB are supported, with end-to-end encryption preserved.

### CLI

The CLI channel requires no external setup:

```bash
skyclaw chat
```

This starts an interactive REPL with readline support, colored Markdown output, streaming responses, and file send/receive via file paths.

## Architecture Overview

SkyClaw is organized as a Cargo workspace with 13 crates plus a binary entry point.

```
                     +------------------+
                     |    skyclaw (bin)  |    CLI entry point (clap)
                     +--------+---------+
                              |
              +---------------+---------------+
              |                               |
   +----------v-----------+       +-----------v----------+
   |  skyclaw-gateway      |       |  skyclaw-agent        |
   |  (SkyGate)            |       |  Agent runtime loop   |
   |  axum HTTP/WS server  |       |  context -> LLM ->    |
   |  routing, sessions,   |       |  tools -> reply       |
   |  rate limiting         |       +-----------+----------+
   +----------+------------+                   |
              |               +----------------+----------------+
              |               |                |                |
   +----------v-----------+  |  +-------------v-+  +-----------v--------+
   |  skyclaw-channels     |  |  |  skyclaw-tools |  |  skyclaw-providers  |
   |  telegram, discord,   |  |  |  shell, file,  |  |  anthropic, openai, |
   |  slack, whatsapp, cli |  |  |  browser, git, |  |  google, mistral,   |
   +----------+------------+  |  |  http, screenshot| |  groq               |
              |               |  +----------------+  +--------------------+
              |               |
   +----------v------------+ |  +----------------+  +--------------------+
   |  skyclaw-memory        | |  |  skyclaw-vault  |  |  skyclaw-filestore  |
   |  sqlite, postgres,     | |  |  ChaCha20-Poly  |  |  local, s3          |
   |  markdown, search      | |  |  vault:// URIs  |  +--------------------+
   +------------------------+ |  +----------------+
                              |
   +------------------------+ |  +----------------+  +--------------------+
   |  skyclaw-skills         | |  |  skyclaw-auto   |  |  skyclaw-observable |
   |  SKILL.md loader,       | |  |  heartbeat,     |  |  tracing, metrics,  |
   |  registry, capability   | |  |  cron scheduler |  |  OpenTelemetry      |
   +-----------+------------+ |  +----------------+  +--------------------+
               |              |
   +-----------v--------------v-------+
   |       skyclaw-core                |
   |  12 traits + shared types +       |
   |  config loading + error types     |
   +-----------------------------------+
```

All crates depend on `skyclaw-core` for trait definitions and shared types. Implementation crates (`skyclaw-channels`, `skyclaw-providers`, etc.) implement traits defined in `skyclaw-core`. The binary crate wires everything together based on configuration.

### Data Flow

1. A message arrives from a messaging platform (Telegram, Discord, etc.)
2. The **Channel** implementation normalizes it into an `InboundMessage`
3. The **Gateway** routes it through session management and rate limiting
4. The **Agent** runtime assembles context (history + memory + skills + system prompt)
5. The **Provider** sends the context to the configured AI model
6. Tool calls in the response are executed by the **Tool** implementations in a sandbox
7. The response streams back through the originating **Channel**
8. The conversation is persisted to the **Memory** backend

## Development Guide

### Building

```bash
# Debug build (fast compilation)
cargo build

# Release build (optimized, LTO)
cargo build --release

# Build without optional features
cargo build --no-default-features

# Build for a specific target
cargo build --release --target x86_64-unknown-linux-musl
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p skyclaw-core

# Run with logging
RUST_LOG=debug cargo test --workspace -- --nocapture
```

### Project Layout

```
skyclaw/
  Cargo.toml              Workspace root
  src/main.rs             Binary entry point (CLI)
  config/default.toml     Default configuration
  crates/
    skyclaw-core/          Traits, types, config, errors
    skyclaw-gateway/       HTTP/WS gateway server
    skyclaw-agent/         Agent runtime loop
    skyclaw-providers/     AI provider implementations
    skyclaw-channels/      Messaging channel implementations
    skyclaw-memory/        Memory backend implementations
    skyclaw-vault/         Encrypted secrets management
    skyclaw-tools/         Built-in tool implementations
    skyclaw-skills/        Skill loading and management
    skyclaw-automation/    Heartbeat and cron
    skyclaw-observable/    Logging, metrics, telemetry
    skyclaw-filestore/     File storage backends
    skyclaw-test-utils/    Shared test utilities
  docs/                    Documentation
  infrastructure/          Terraform and Fly.io configs
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `telegram` | on | Telegram channel support |
| `discord` | on | Discord channel support |
| `slack` | on | Slack channel support |
| `whatsapp` | on | WhatsApp channel support |
| `browser` | on | Browser automation tool (requires Chrome/Chromium) |
| `postgres` | on | PostgreSQL memory backend |

Disable features to reduce binary size:

```bash
cargo build --release --no-default-features --features telegram
```

### Contributing

1. Fork the repository.
2. Create a feature branch: `git checkout -b feat/my-feature`.
3. Write tests for new functionality.
4. Ensure `cargo test --workspace` passes.
5. Ensure `cargo clippy --workspace` reports no warnings.
6. Format code with `cargo fmt --all`.
7. Submit a pull request.

## License

MIT -- see [LICENSE](LICENSE) for details.
