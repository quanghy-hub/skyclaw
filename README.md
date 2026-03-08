<p align="center">
  <img src="assets/banner.png" alt="SkyClaw" width="100%">
</p>

# SkyClaw

Cloud-native Rust AI agent runtime. Telegram-native. One binary, zero config files.

## What It Does

SkyClaw is an autonomous AI agent that lives on your server and talks to you through Telegram. It can run shell commands, browse the web, read and write files, and fetch URLs — all controlled through natural conversation.

No web dashboards. No config files to edit. Just deploy, paste your API key in Telegram, and go.

## 3-Step Setup

### Step 1: Get a Telegram Bot Token

1. Open Telegram and search for [@BotFather](https://t.me/BotFather)
2. Send `/newbot`
3. Choose a name (e.g. "My SkyClaw") and a username (must end in `bot`, e.g. `my_skyclaw_bot`)
4. BotFather replies with your bot token — looks like `123456789:ABCdefGHI-jklMNOpqrSTUvwxYZ`
5. Copy it

### Step 2: Deploy

```bash
git clone https://github.com/nagisanzenin/skyclaw.git
cd skyclaw
cargo build --release
export TELEGRAM_BOT_TOKEN="123456789:ABCdefGHI-jklMNOpqrSTUvwxYZ"
./target/release/skyclaw start
```

### Step 3: Activate

1. Open your bot in Telegram (search for the username you chose)
2. Send any message — SkyClaw asks for your API key
3. Paste your key (Anthropic, OpenAI, or Gemini)
4. SkyClaw validates it against the real API and goes online

## Supported Providers

Paste any of these API keys in Telegram — SkyClaw detects the provider automatically:

| Key Pattern | Provider | Default Model |
|------------|----------|---------------|
| `sk-ant-*` | Anthropic | claude-sonnet-4-6 |
| `sk-*` | OpenAI | gpt-5.2 |
| `AIzaSy*` | Google Gemini | gemini-3-flash-preview |

## What SkyClaw Can Do

| Tool | Description |
|------|-------------|
| **Shell** | Run any command on your server |
| **Browser** | Headless Chrome — navigate, click, type, screenshot, extract text |
| **File ops** | Read, write, list files on the server |
| **Web fetch** | HTTP GET with token-budgeted response extraction |
| **Messaging** | Send real-time updates during multi-step tasks |

## Self-Configuration

SkyClaw can change its own settings through natural language. Just tell it:

- "Change model to claude-opus-4-6"
- "Switch to GPT-5.2"

Config lives at `~/.skyclaw/credentials.toml` — SkyClaw reads and edits this file itself.

## Security

- **Auto-whitelist**: The first person to message the bot gets whitelisted. Everyone else is denied.
- **No open access**: Empty allowlist = deny all. No one can use the bot until the first user claims it.
- **Numeric ID only**: Allowlist matches on Telegram user IDs, not usernames (usernames can be changed).

## Architecture

13-crate Cargo workspace:

```
skyclaw (binary)
├── skyclaw-core         Traits, types, config, errors
├── skyclaw-gateway      HTTP server, health endpoint
├── skyclaw-agent        Agent runtime loop (context → LLM → tools → reply)
├── skyclaw-providers    Anthropic, OpenAI-compatible, Google Gemini
├── skyclaw-channels     Telegram
├── skyclaw-memory       SQLite persistent memory
├── skyclaw-vault        ChaCha20-Poly1305 encrypted secrets
├── skyclaw-tools        Shell, browser, file ops, web fetch
├── skyclaw-skills       Skill registry
├── skyclaw-automation   Heartbeat, cron scheduler
├── skyclaw-observable   Tracing, OpenTelemetry
├── skyclaw-filestore    Local and S3 file storage
└── skyclaw-test-utils   Test helpers
```

## CLI Reference

```
skyclaw start              Start the gateway daemon
skyclaw chat               Interactive CLI chat
skyclaw status             Show running state
skyclaw config validate    Validate configuration
skyclaw config show        Print resolved config
skyclaw version            Show version info
```

## Development

```bash
cargo check --workspace          # Quick compilation check
cargo build --workspace          # Debug build
cargo test --workspace           # Run all tests
cargo clippy --workspace -- -D warnings   # Lint
cargo fmt --all                  # Format
cargo build --release            # Release build
```

## Requirements

- Rust 1.82+
- Chrome/Chromium (for browser tool)
- A Telegram bot token

## License

MIT
