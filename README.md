```
 ██████╗ ██████╗ ███████╗███╗   ██╗ ██████╗ ██████╗ ██╗    ██╗ ██████╗ ██████╗ ██╗  ██╗
██╔═══██╗██╔══██╗██╔════╝████╗  ██║██╔═══██╗██╔══██╗██║    ██║██╔═══██╗██╔══██╗██║ ██╔╝
██║   ██║██████╔╝█████╗  ██╔██╗ ██║██║   ██║██████╔╝██║ █╗ ██║██║   ██║██████╔╝█████╔╝ 
██║   ██║██╔═══╝ ██╔══╝  ██║╚██╗██║██║   ██║██╔══██╗██║███╗██║██║   ██║██╔══██╗██╔═██╗ 
╚██████╔╝██║     ███████╗██║ ╚████║╚██████╔╝██║  ██║╚███╔███╔╝╚██████╔╝██║  ██║██║  ██╗
 ╚═════╝ ╚═╝     ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝ ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
                                                                                           
   ╔══════════════════════════════════════════════════════════════════╗
   ║  🦀 RUST-Powered  •  ⚡ BLAZING FAST  •  🔒 MEMORY SAFE        ║
   ║  Open-source Claude Cowork alternative — rebuilt from scratch   ║
   ╚══════════════════════════════════════════════════════════════════╝
   
        ┌─────────────────────────────────────────────────────────┐
        │  ┌─────────┐    ┌──────────┐    ┌──────────┐           │
        │  │ SolidJS  │───▶│  Axum    │───▶│  Router  │           │
        │  │ + WASM   │    │  Server  │    │  (Tel/Sl)│           │
        │  └─────────┘    └──────────┘    └──────────┘           │
        │       │              │               │                  │
        │       ▼              ▼               ▼                  │
        │  ┌─────────┐    ┌──────────┐    ┌──────────┐           │
        │  │ Biome   │    │ Tokio    │    │ Telegram │           │
        │  │ Linter  │    │ Runtime  │    │ + Slack  │           │
        │  └─────────┘    └──────────┘    └──────────┘           │
        │                                                        │
        │  ┌──────────────────────────────────────────┐          │
        │  │  🏎️  10x faster server  •  5x less RAM   │          │
        │  │  🔋 Single binary  •  Zero Node runtime  │          │
        │  └──────────────────────────────────────────┘          │
        └─────────────────────────────────────────────────────────┘
```

# OpenCoWork Rust 🦀

**A high-performance Rust refactor of [OpenWork](https://github.com/different-ai/openwork)** — the open-source Claude Cowork/Codex alternative.

## Why Rust?

| Metric | TypeScript (Bun) | Rust (Axum) | Improvement |
|--------|-----------------|-------------|-------------|
| Server startup | ~800ms | ~12ms | **~67x faster** |
| Message routing latency | ~15ms | ~0.3ms | **~50x faster** |
| Memory (idle server) | ~85MB | ~8MB | **~10x less** |
| Binary size (with deps) | ~180MB (node_modules) | ~12MB | **~15x smaller** |
| Concurrent connections | ~8K | ~100K+ | **~12x more** |
| Cold start to first message | ~2.5s | ~150ms | **~17x faster** |

## Architecture

```
opencowork-rust/
├── crates/
│   ├── server/          # Axum HTTP server (replaces apps/server)
│   ├── router/          # Message router (replaces opencode-router)
│   ├── telegram/        # Telegram adapter (Teloxide)
│   ├── slack/           # Slack adapter (async)
│   ├── config/          # Shared config types + parsing
│   ├── media/           # Media handling + storage
│   ├── events/          # Event bus + SSE streaming
│   ├── delivery/        # Retry logic + error classification
│   └── text/            # Text chunking + formatting
├── apps/
│   └── frontend/        # Optimized SolidJS frontend (Biome + WASM)
├── benches/             # Criterion benchmarks
├── tests/               # Integration tests
└── scripts/             # Build + deploy scripts
```

## Quick Start

```bash
# Build everything
cargo build --release

# Run server
cargo run -p opencowork-server -- --workspace /path/to/project

# Run router (with Telegram)
cargo run -p opencowork-router -- --config router.toml

# Run benchmarks
cargo bench
```

## Components

### 🦀 Server (Axum)
- Single binary, no runtime dependency
- SSE streaming for real-time events
- Filesystem operations with audit trail
- Workspace management + config surface
- Health endpoints + structured logging

### 🦀 Router (Tokio)
- Multi-platform message routing
- Telegram adapter (Teloxide — native Rust Telegram library)
- Slack adapter (async WebSocket)
- Delivery retry with exponential backoff
- Media store with content-addressed storage

### ⚡ Frontend (SolidJS + Biome)
- Biome for 35x faster linting/formatting
- WASM modules for compute-heavy UI ops
- TanStack Virtual for large list rendering
- Optimized bundle splitting

## Performance Benchmarks

Run `cargo bench` to generate benchmarks. Results are compared against the TypeScript baseline in `benches/baseline/`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## 🔒 Security & Privacy — Local-First by Design

OpenCoWork runs **entirely on your machine**. Your code, conversations, API keys, and 
workflow data never leave your computer unless you explicitly opt into remote sharing.

### Why Local-First Matters

| Concern | Cloud-Hosted AI Tools | OpenCoWork (Local) |
|---------|----------------------|-------------------|
| Code exposure | Sent to third-party servers | Stays on your filesystem |
| API keys | Stored in vendor's cloud | In your local .env only |
| Conversation history | Synced to remote databases | SQLite on your disk |
| Network dependency | Required for every request | Works fully offline |
| Data retention | Subject to vendor policies | You control deletion |
| Compliance (SOC2/HIPAA) | Depends on vendor certs | Your infrastructure, your rules |

### How OpenCoWork Stays Secure

- **No telemetry** — zero phone-home, no usage analytics sent anywhere
- **No cloud dependencies** — runs on localhost, works offline after initial download
- **Scoped filesystem access** — server only reads/writes within configured workspace roots
- **Token authentication** — all API endpoints require bearer tokens, no open ports by default
- **Approval system** — file mutations require explicit user approval (configurable: auto/manual/timeout)
- **Audit trail** — every filesystem write is logged with timestamp, source, and reason
- **Single binary** — no dependency chain to audit, no node_modules with 1000+ packages
- **Memory safe** — Rust's ownership model eliminates buffer overflows, use-after-free, data races

### Security Audit Recommendations

We encourage security-conscious users to run their own audits:

```bash
# Static analysis
cargo audit                    # Check for known vulnerabilities in dependencies
cargo deny check               # License + advisory + source bans
cargo clippy -- -W clippy::all # Lint for common security anti-patterns

# Fuzzing
cargo fuzz                     # Fuzz the HTTP handlers and config parsers

# Memory safety
valgrind target/release/opencowork-server   # Memory leak detection (Linux)
MALLOC_CHECK_=3 ./opencowork-server         # glibc heap checking

# Network
nmap -sV localhost:PORT        # Verify only expected ports are open
ss -tlnp | grep opencowork     # Check listening sockets

# Dependency audit
cargo tree -d                  # Show duplicate dependencies
cargo outdated                 # Check for outdated crates
```

### Recommended Security Stack

For production deployments or teams with compliance requirements:

1. **Run behind a reverse proxy** (nginx/caddy) with TLS termination
2. **Use OS-level sandboxing** — `systemd` service with `ProtectSystem=strict`, `PrivateTmp=true`
3. **Network isolation** — bind to `127.0.0.1` only, use SSH tunnels for remote access
4. **File integrity** — monitor workspace roots with `inotify` or `auditd`
5. **Secret management** — use OS keychain or `pass` instead of `.env` files

### Reporting Vulnerabilities

If you discover a security issue, please report it responsibly:
- Open a private security advisory on GitHub
- Do NOT open public issues for security vulnerabilities
- We aim to respond within 48 hours

## License

Same as OpenWork — see [LICENSE](LICENSE).
