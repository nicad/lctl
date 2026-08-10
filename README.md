# Angel

The launchctl macOS users deserve.

A friendlier CLI for managing launchd services — install, start/stop/restart, status, logs, and more.

## Install

```bash
cargo install --path .
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
