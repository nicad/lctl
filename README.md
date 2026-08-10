# Angel

The launchctl macOS *should've* implemented.

A friendly wrapper for launchctl, complete with a status command.

Inspired by [lunchy](https://github.com/eddiezane/lunchy), but using the new launchctl API, and with a focus on features.

**Features:**
- Services can live wherever you want
- More meaningful errors
- A status command!
- Shell completion
- Fuzzy and regex search
- Beautiful and customizable search output
- More to come!

## Install

```bash
brew install oliviaBahr/tap/angel
```

### From source (contributors)

```bash
cargo install --path .
```
Installs the binary to `~/.cargo/bin` and the man page to `~/.cargo/share/man/man1/`.

## Usage

```bash
angel list
angel install ./my.service.plist
angel start my.service
angel status my.service
angel logs my.service
angel stop my.service
angel uninstall my.service

angel reboot my.service      # bootout + bootstrap (reload plist from disk)
angel bootstrap my.service
angel bootout my.service
angel enable my.service
angel disable my.service
angel print my.service       # launchctl print wrapper
angel plist my.service       # print the installed plist
```

Common flags:

- `-v` / `--verbose` — verbose output
- `-y` / `--yes` — assume yes for confirmation prompts
- `-e` / `--exact` — exact service name match (no fuzzy)
- `install --domain gui|user|system` — skip interactive domain selection
- `install -s symlink|copy|move` — how to place the plist (default: symlink)
- `logs -t 5m` — log time frame (e.g. `1h`, `24h`)

Run `angel --help` or `angel <command> --help` for options. See `man angel` for the full manual.

Shell completions install automatically via Homebrew. From a cargo build:

```bash
angel completion zsh  > "${fpath[1]}/_angel"   # or another dir on $fpath
angel completion bash > "$(brew --prefix)/etc/bash_completion.d/angel"
angel completion fish > ~/.config/fish/completions/angel.fish
```

## Configuration

Angel optionally reads YAML from the first existing path among:

- `~/.angelrc`
- `$XDG_CONFIG_HOME/angel/.angelrc`
- `~/.config/angel/.angelrc`

Example:

```yaml
directories:
  - path: ~/Services
    domain: gui
  - path: /Library/LaunchDaemons
    domain: system
```

Valid `domain` values: `gui`, `user`, `system`. Paths may use `~`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, hooks, and tests.
