# Contributing

## Setup

```bash
git config core.hooksPath .githooks
cargo install --path .
```

Pre-commit runs `cargo fmt --check --all` and blocks the commit if formatting fails. Fix with `cargo fmt`.

## Tests

```bash
# unit + GUI e2e, no sudo
make test

# everything, including system e2e (asks for sudo; run from a real terminal)
make test-all

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

`e2e-all` includes interactive auto-elevate tests that run `sudo -K` then plain `lctl` with `/dev/tty` stdin — expect mid-suite password prompts. Run from a real terminal (not a pipe).

Aliases live in `.cargo/config.toml`. Do **not** `sudo cargo test` — suite must run as your user (GUI / `$HOME`); only individual `lctl` / `launchctl` calls are elevated.

Unit tests live in [`tests/unit/`](tests/unit/) (no launchd). E2e lives in [`tests/e2e/`](tests/e2e/) and uses real launchd; default install strategy is symlink. Cleanup uses raw `launchctl` / `rm`, not lctl. System install asserts LaunchDaemons symlink + fixture are `root:wheel` mode `644`.
