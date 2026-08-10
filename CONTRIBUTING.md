# Contributing

## Setup

```bash
git config core.hooksPath .githooks
cargo install --path .
```

Pre-commit runs `cargo fmt --check --all` and blocks the commit if formatting fails. Fix with `cargo fmt`.

Users install with `brew install oliviaBahr/tap/angel` (see [README.md](README.md)).

## Tests

```bash
# unit + GUI e2e
cargo test

# unit only
cargo test --test unit
# or: cargo unit

# e2e GUI only (system tests ignored)
cargo e2e

# GUI + system e2e (prompts for sudo in your shell, then runs tests)
sudo -v && cargo e2e-all
```

`e2e-all` includes interactive auto-elevate tests that run `sudo -K` then plain `angel` with `/dev/tty` stdin — expect mid-suite password prompts. Run from a real terminal (not a pipe).

Aliases live in `.cargo/config.toml`. Do **not** `sudo cargo test` — suite must run as your user (GUI / `$HOME`); only individual `angel` / `launchctl` calls are elevated.

Unit tests live in [`tests/unit/`](tests/unit/) (no launchd). E2e lives in [`tests/e2e/`](tests/e2e/) and uses real launchd; default install strategy is symlink. Cleanup uses raw `launchctl` / `rm`, not angel. System install asserts LaunchDaemons symlink + fixture are `root:wheel` mode `644`.

## Releasing

1. Ensure `version` in `Cargo.toml` is the version you want to ship.
2. Push `main`.
3. On GitHub: **Actions → Release → Run workflow**, enter the version (e.g. `0.1.0`).
4. The workflow tags `vX.Y.Z`, creates a GitHub Release, and updates [`oliviaBahr/homebrew-tap`](https://github.com/oliviaBahr/homebrew-tap) `Formula/angel.rb`.

Requires repo secret `HOMEBREW_TAP_TOKEN`: a fine-grained or classic PAT that can push to `oliviaBahr/homebrew-tap`.
