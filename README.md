# lctl

A simpler CLI on top of macOS `launchctl`.

Fork of [angel](https://github.com/oliviaBahr/angel) by Olivia Bahr (MIT), renamed and maintained separately.

- Fuzzy (substring) service name matching, with a picker when several services match
- `install` / `uninstall` that place the plist and load it in one step
- A `status` command that renders `launchctl print` as a table
- `logs` that combines the plist's stdout/stderr files with the unified log
- Auto-retry with `sudo` when launchctl needs root
- Shell completion and a man page

lctl makes no network calls. It only runs `/bin/launchctl`, `/usr/bin/log`, `/usr/bin/sudo`, and (under sudo, for system installs) `ln`, `mv`, `cp`, `rm`, `chown`, `chmod`, all by absolute path.

## Install

```bash
cargo install --path .
```

Installs the binary to `~/.cargo/bin` and the man page to `~/.cargo/share/man/man1/`.

Shell completions:

```bash
lctl completion zsh  > "${fpath[1]}/_lctl"   # or another dir on $fpath
lctl completion bash > "$(brew --prefix)/etc/bash_completion.d/lctl"
lctl completion fish > ~/.config/fish/completions/lctl.fish
```

## Usage

```bash
lctl list
lctl install ./my.service.plist
lctl start my.service
lctl status my.service
lctl logs my.service
lctl stop my.service
lctl uninstall my.service

lctl reboot my.service      # bootout + bootstrap (reload plist from disk)
lctl bootstrap my.service
lctl bootout my.service
lctl enable my.service
lctl disable my.service
lctl print my.service       # raw launchctl print
lctl plist my.service       # print the plist file
```

Common flags:

- `-v` / `--verbose` — verbose output, including every command lctl executes
- `-y` / `--yes` — assume yes for confirmation prompts (and pick the first match when several services match)
- `-e` / `--exact` — exact (case-insensitive) service name match instead of substring
- `install --domain gui|user|system` — skip interactive domain selection
- `install -s symlink|copy|move` — how to place the plist (default: symlink)
- `logs -t 5m` — log time frame (e.g. `1h`, `24h`)
- `list -a` / `-d` / `-i` — also show Apple services / services without a plist on disk / services that are not running

Run `lctl --help` or `lctl <command> --help` for options. See `man lctl` for the full manual.

## How lctl maps to launchctl

Run any command with `-v` to see the exact launchctl invocations.

### launchctl concepts

- **Domain** — where a service lives in launchd:
  - `system` — LaunchDaemons, run as root at boot. Modifying it requires root.
  - `gui/<uid>` — LaunchAgents for a logged-in (Aqua) user session.
  - `user/<uid>` — LaunchAgents for a background (non-GUI, e.g. SSH) session of that user.
- **Service target** — `<domain>/<label>`, e.g. `gui/501/com.example.foo`. `<label>` is the plist's `Label`.
- **Loaded (bootstrapped)** — launchd knows about the service. Loading may start it (`RunAtLoad`, `KeepAlive`, sockets, timers…).
- **Running** — the service currently has a process (a PID). A loaded service can be idle.
- **Disabled** — a persistent override stored by launchd, independent of the plist file. It survives `bootout` and reboots, and stops the service from being loaded or started until `enable` clears it.
- A plist in `~/Library/LaunchAgents`, `/Library/LaunchAgents` or `/Library/LaunchDaemons` is loaded again automatically at the next login/boot, even after `bootout`, unless it is disabled or removed.

### lctl → launchctl

`<target>` is `<domain>/<label>`; `<plist>` is the service's plist path.

| lctl | launchctl (and other) commands executed | Notes |
|---|---|---|
| `install <file>` | place `<file>` at `~/Library/LaunchAgents/<label>.plist` (gui/user) or `/Library/LaunchDaemons/<label>.plist` (system) by symlink/copy/move<br>system only: `chown -h root:wheel` + `chmod 644` (via sudo)<br>if already running: `launchctl disable <target>`, `launchctl bootout <target>`, `launchctl enable <target>`<br>`launchctl bootstrap <domain> <plist>` | Asks before overwriting or killing. With the default symlink strategy and the system domain, your source file itself becomes `root:wheel 644` (launchd requires it). |
| `uninstall <name>` | `launchctl bootout <target>`<br>delete `<plist>`<br>`launchctl enable <target>` | If `<plist>` is a symlink, only the link is removed. `enable` clears any leftover disabled override. |
| `start <name>` (alias `kickstart`) | `launchctl kickstart <target>` | Service must already be loaded. |
| `start -k <name>` | `launchctl kickstart -k <target>` | Kills the running instance first. |
| `restart <name>` (alias `kkill`) | `launchctl kickstart -k <target>` | Same as `start -k`. |
| `stop <name>` (alias `kill`) | `launchctl kill sigterm <target>` | `-s sigkill\|sighup\|sigint` to change the signal. With `KeepAlive`, launchd starts it again; use `bootout` or `disable` to keep it down. |
| `reboot <name>` | `launchctl bootout <target>`<br>`launchctl bootstrap <domain> <plist>` | Reloads the plist from disk (picks up edits and symlink target changes). |
| `bootstrap <name>` | `launchctl bootstrap <domain> <plist>` | Needs a plist on disk. |
| `bootout <name>` | `launchctl bootout <target>` | Stops and unloads. Comes back at next login/boot if the plist is in a standard Launch* directory. |
| `enable <name>` | `launchctl enable <target>` | |
| `disable <name>` | `launchctl disable <target>` | Does not stop a running instance. |
| `print <name>` | `launchctl print <target>` | Raw output. |
| `status <name>` | `launchctl print <target>` | Parsed and shown as a table. |
| `list [pattern]` | none beyond the discovery step below | Reads plists from the directories listed under [Configuration](#configuration). |
| `logs <name>` | read last 50 lines of the plist's `StandardOutPath` / `StandardErrorPath`<br>`log show --predicate 'process == "<program>"' --last 5m --style compact` | `<program>` is the basename of `Program` / `ProgramArguments[0]`. |
| `plist <name>` | read `<plist>` | |

### launchctl → lctl

| launchctl | Modern equivalent | lctl |
|---|---|---|
| `launchctl load <plist>` | `launchctl bootstrap <domain> <plist>` | `lctl bootstrap <name>` (plist already in place) or `lctl install <file>` (place + load) |
| `launchctl load -w <plist>` | `launchctl enable <target>` + `launchctl bootstrap <domain> <plist>` | `lctl enable <name>` then `lctl bootstrap <name>` |
| `launchctl unload <plist>` | `launchctl bootout <target>` | `lctl bootout <name>` |
| `launchctl unload -w <plist>` | `launchctl disable <target>` + `launchctl bootout <target>` | `lctl disable <name>` then `lctl bootout <name>` |
| `launchctl remove <label>` | `launchctl bootout <target>` | `lctl bootout <name>` |
| `launchctl start <label>` | `launchctl kickstart <target>` | `lctl start <name>` |
| `launchctl stop <label>` | `launchctl kill SIGTERM <target>` | `lctl stop <name>` |
| `launchctl list` | `launchctl print <domain>` | `lctl list` |
| `launchctl list <label>` | `launchctl print <target>` | `lctl status <name>` or `lctl print <name>` |
| `launchctl kickstart -k <target>` | — | `lctl restart <name>` |
| `launchctl kill <sig> <target>` | — | `lctl stop -s <sig> <name>` |
| `launchctl enable` / `disable <target>` | — | `lctl enable` / `lctl disable <name>` |
| `launchctl print-disabled <domain>` | — | not wrapped; use launchctl |
| `launchctl blame <target>` | — | not wrapped; use launchctl |
| `launchctl submit`, `setenv`, `getenv`, `asuser`, `bsexec`, `limit`, `config`, `attach`, `debug`, `procinfo`, `dumpstate`, … | — | not wrapped; use launchctl |

The legacy subcommands (`load`, `unload`, `start`, `stop`, `list`, `remove`) infer the domain from who runs them: as root they target `system`, otherwise the current user's domain. lctl always passes an explicit domain.

### What lctl does around those commands

- **Discovery.** Every command except `completion` first runs `launchctl print system`, `launchctl print user/<uid>` and `launchctl print gui/<uid>`, and reads the plist directories, to build the list of known services.
- **Name resolution.** `<name>` is a case-insensitive substring of the label (`-e` for an exact match). If several services match, lctl shows a picker (`-y` picks the first alphabetically).
- **Domain resolution.** A service's domain is, in order of precedence: the domain where launchd currently has it loaded; the plist's `LimitLoadToSessionType` (`Aqua` → `gui`, `Background`/`LoginWindow` → `user`, `System` → `system`); the domain of the directory the plist was found in.
- **sudo.** lctl runs launchctl as you first. If launchctl reports it needs root (`Operation not permitted` or `Try re-running the command as root`) and stdin is a terminal, lctl runs `sudo -v` (password prompt) and retries with `sudo -n /bin/launchctl …`. Installing into the `system` domain prompts for sudo up front. Running lctl itself under `sudo` also works: it uses `SUDO_UID` to find your home and `gui/<uid>`.

## Configuration

Directories scanned for plists:

| Directory | Domain | In `lctl list` by default |
|---|---|---|
| `/System/Library/LaunchDaemons` | `system` | no (Apple; `-a`) |
| `/System/Library/LaunchAgents`, `/System/Library/LaunchAngels` | `gui/<uid>` | no (Apple; `-a`) |
| `/Library/LaunchDaemons` | `system` | yes |
| `/Library/LaunchAgents`, `~/Library/LaunchAgents` | `gui/<uid>` | yes |
| `~/Library/Application Support/lctl/{gui,user,system}` | per subdirectory | yes |
| `directories` from the config file | per entry | yes |

The lctl directory can be moved with `LCTL_CONFIG_DIR`. `lctl install <name>` also accepts the name of a plist found in its `{gui,user,system}` subdirectories instead of a path.

lctl optionally reads YAML from the first existing path among:

- `~/.lctlrc`
- `~/Library/Application Support/lctl/.lctlrc`
- `~/.config/lctl/.lctlrc`

Example:

```yaml
directories:
  - path: ~/Services
    domain: gui
  - path: /Library/LaunchDaemons
    domain: system
```

Valid `domain` values: `gui`, `user`, `system`. Paths may use `~`. The config is skipped when running in a root shell (home is `/var/root`).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, hooks, and tests.

## License

MIT — see [LICENSE](LICENSE). Original work © Olivia Bahr.
