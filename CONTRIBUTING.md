# Contributing to OpenCoWork Rust

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone
git clone https://github.com/multidimensionalinteractive/opencowork-rust.git
cd opencowork-rust

# Build
cargo build

# Test
cargo test

# Lint
cargo clippy -- -W clippy::all

# Frontend
cd apps/frontend
bun install
bun dev
```

## Project Structure

```
crates/
  server/    — Axum HTTP server
  router/    — Message routing engine
  telegram/  — Telegram adapter (Teloxide)
  slack/     — Slack adapter
  config/    — Shared configuration types
  events/    — Event bus + SSE
  media/     — Media handling
  delivery/  — Retry + error classification
  text/      — Text chunking + formatting
apps/
  frontend/  — SolidJS UI with Biome
```

## Code Style

- `cargo fmt` for formatting
- `cargo clippy` for linting
- Doc comments on all public items
- `#[inline]` on hot-path functions
- Prefer `Arc<T>` over `Box<T>` for shared state
- Use `DashMap` for concurrent mutable state

## Performance Guidelines

- Benchmark before and after any change to hot paths
- Use `criterion` for microbenchmarks
- Profile with `perf` or `flamegraph` for bottlenecks
- Avoid allocations in message routing (pre-allocate buffers)
- Prefer `&str` over `String` in function signatures
- Use `SmallVec` for small collections (< 8 items)

## Pull Requests

1. Fork the repo
2. Create a feature branch
3. Write tests for new functionality
4. Run `cargo test && cargo clippy`
5. Submit PR with benchmark results if touching hot paths
