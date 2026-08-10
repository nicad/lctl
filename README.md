# Angel

The launchctl macOS users deserve.

A friendlier CLI for managing launchd services — install, start/stop/restart, status, logs, and more.

## Install

```bash
cargo install --path .
```

Installs the binary to `~/.cargo/bin` and the man page to `~/.cargo/share/man/man1/`.
If Homebrew (or anything else) sets `MANPATH`, add cargo's man dir so `man angel` works:

```bash
export MANPATH="$HOME/.cargo/share/man${MANPATH:+:$MANPATH}"
```

Requires macOS.

## Usage

```bash
angel list
angel install ./my.service.plist
angel start my.service
angel status my.service
angel logs my.service
angel stop my.service
angel uninstall my.service
```

Run `angel --help` or `angel <command> --help` for options.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, hooks, and tests.
