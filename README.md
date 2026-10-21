# lctl

A simpler CLI on top of macOS `launchctl`.

Fork of [angel](https://github.com/oliviaBahr/angel) by Olivia Bahr (MIT), renamed and maintained separately.

- Fuzzy (substring) service name matching, with a picker when several services match
- Commands named after what you want (`start`, `stop`, `enable`, `disable`, `reload`) that run the needed launchctl steps
- `install` / `uninstall` that place the plist and load it in one step
- A `status` command that renders `launchctl print` as a table
- `logs` that combines the plist's stdout/stderr files with the unified log
- Auto-retry with `sudo` when launchctl needs root
- Shell completion and a man page

lctl makes no network calls. It runs `/bin/launchctl`, `/usr/bin/log` (for `logs`) and `/usr/bin/sudo`. For `system`-domain installs without root it also runs `/bin/ln`, `/bin/cp`, `/bin/mv`, `/bin/rm`, `/usr/sbin/chown` and `/bin/chmod` under `sudo -n`. All by absolute path.

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

lctl restart my.service
lctl reload my.service      # pick up plist edits
lctl disable my.service     # stop it and keep it off across reboots
lctl enable my.service      # undo disable, and load it
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

Run any command with `-v` to see each command lctl executes.

### launchctl concepts

- **Domain** — where a service lives in launchd:
  - `system` — LaunchDaemons, run as root at boot. Modifying it requires root.
  - `gui/<uid>` — LaunchAgents for a logged-in (Aqua) user session.
  - `user/<uid>` — LaunchAgents for a background (non-GUI, e.g. SSH) session of that user.
- **Service target** — `<domain>/<label>`, e.g. `gui/501/com.example.foo`. `<label>` is the plist's `Label`.
- **Loaded (bootstrapped)** — launchd has read the plist and manages the service. Loading starts the process only if the plist asks for it (`RunAtLoad`, `KeepAlive`, `StartInterval`, sockets…).
- **Running** — the service currently has a process (a PID). A loaded service can be idle.
- **Disabled** — a flag launchd stores in its own database, not in the plist. While it is set, the service cannot be loaded (`bootstrap` fails). It survives `bootout` and reboots until `enable` clears it. Disabling a service that is already loaded does not unload or stop it.
- **Unloaded (booted out)** — launchd forgets the service and stops its process: SIGTERM, then SIGKILL after the plist's `ExitTimeOut`. The plist file stays on disk. A plist in `~/Library/LaunchAgents`, `/Library/LaunchAgents` or `/Library/LaunchDaemons` is loaded again at the next login/boot unless it is disabled or removed.

### lctl → launchctl

`<target>` is `<domain>/<label>`; `<plist>` is the service's plist path. "Loaded" means the service was listed by the discovery step (see [below](#what-lctl-does-around-those-commands)) when the command started.

Steps run in order. lctl stops at the first step that fails and prints launchctl's error, except for the steps of `uninstall` marked best-effort.

| lctl | Executes, in order | Notes |
|---|---|---|
| `start <name>` | 1. if not loaded: `launchctl bootstrap <domain> <plist>`<br>2. `launchctl kickstart <target>` | `kickstart` runs the process now, whatever the plist's launch conditions. An already running process is left alone. A disabled service can't be loaded: run `enable` first. |
| `stop <name>` | if loaded: `launchctl bootout <target>` | If it isn't loaded, nothing runs. Because the service is unloaded, `KeepAlive` can't restart it. It comes back at the next login/boot if the plist is in a standard Launch* directory; `disable` prevents that. |
| `restart <name>` | if loaded: `launchctl kickstart -k <target>`<br>if not loaded: 1. `launchctl bootstrap <domain> <plist>` 2. `launchctl kickstart <target>` | `-k` kills the running process first, if there is one. When not loaded it behaves like `start`. |
| `reload <name>` | 1. if loaded: `launchctl bootout <target>`<br>2. `launchctl bootstrap <domain> <plist>` | Picks up plist edits and symlink target changes. Fails before unloading if the service has no plist on disk. |
| `enable <name>` | 1. `launchctl enable <target>`<br>2. if not loaded: `launchctl bootstrap <domain> <plist>` | The flag change persists across reboots. Loading runs the process only if the plist asks for it; `start` runs it now. |
| `disable <name>` | 1. `launchctl disable <target>`<br>2. if loaded: `launchctl bootout <target>` | Persists across reboots until `enable`. Disabling before the bootout keeps `KeepAlive` from restarting it. The plist stays on disk. |
| `status <name>` | if loaded: `launchctl print <target>` | Parsed into a table. If not loaded, shows "not loaded" and the plist path. `-d` also prints the raw `launchctl print` output and the parsed data. |
| `list [pattern]` | nothing beyond discovery | Columns: last exit code, PID, domain, label, plist path. By default only services that are running, have a plist on disk and are not Apple's; `-i`, `-d`, `-a` lift each filter. "Apple" means a plist under `/System/Library`, or, for a service without a plist, a label containing `com.apple`. |
| `logs <name>` | 1. print the last 50 lines of the plist's `StandardOutPath` and `StandardErrorPath` (those that exist)<br>2. `/usr/bin/log show --predicate 'process == "<program>"' --last <time> --style compact`, last 50 lines | `<program>` is the file name of `Program`, else of `ProgramArguments[0]`, else the label. `<time>` comes from `-t` (default `5m`). No launchctl call. |
| `plist <name>` | read `<plist>` and print it | Text (XML) plists only; a binary plist fails (use `plutil -p`). |
| `install <file>` | see [install](#install-1) | |
| `uninstall <name>` | see [uninstall](#uninstall) | |
| `completion <shell>` | nothing (no discovery either) | Prints a shell completion script. |

#### install

`lctl install <file>`:

1. **Source.** `<file>` is a path; symlinks are resolved to the real file. If no file exists there, `<file>` is treated as a service name and looked up among the plists in `~/Library/Application Support/lctl/{gui,user,system}`. The plist must have a `Label`.
2. **Domain.** Taken from `--domain`; else from the `{gui,user,system}` subdirectory the source is in; else lctl asks (default `gui`). If the plist sets `LimitLoadToSessionType`, the two must agree (`Aqua` = `gui`, `Background`/`LoginWindow` = `user`, `System` = `system`), or lctl stops. For `system`, lctl runs `sudo -v` at this point.
3. **Destination.** `~/Library/LaunchAgents/<label>.plist` for `gui`/`user`, `/Library/LaunchDaemons/<label>.plist` for `system`. If something is already there, lctl asks (default yes) and removes it.
4. **Place.** `-s symlink` (default) creates a symlink to the source; `-s copy` copies it; `-s move` moves it.
5. **Ownership (`system` only).** launchd only loads root-owned daemon plists. lctl runs `chown -h root:wheel` and `chmod 644` on the source file (symlink, move) and on the placed file (symlink, copy). With the default symlink, your source file becomes `root:wheel 644`.
6. **Existing service.** If a service with that label is loaded, lctl asks (default no), then runs `launchctl disable <target>`, `launchctl bootout <target>`, `launchctl enable <target>`.
7. `launchctl bootstrap <domain> <plist>`

For `system` when lctl isn't running as root, steps 3–5 run through `sudo -n`: `/bin/rm -f`, `/bin/ln -sf`, `/bin/cp`, `/bin/mv`, `/usr/sbin/chown -h root:wheel`, `/bin/chmod 644`. As root, lctl makes the same changes directly.

#### uninstall

`lctl uninstall <name>`:

1. Asks for confirmation (default no; `-y` skips it).
2. `launchctl bootout <target>`. Best-effort: a failure (for example, not loaded) is ignored and shown only with `-v`.
3. Deletes `<plist>`: the plist found during discovery or, for a service without one, the `path` reported by `launchctl print <target>`. If it is a symlink, only the link is deleted, not the file it points to. This step does not use sudo, so for a plist in `/Library/LaunchDaemons` run `sudo lctl uninstall <name>`.
4. `launchctl enable <target>`. Best-effort: clears a disabled flag so a later install of the same label can load. A failure prints a warning.

### launchctl → lctl

| launchctl | Closest modern equivalent | lctl |
|---|---|---|
| `launchctl load <plist>` | `launchctl bootstrap <domain> <plist>` | `lctl enable <name>` (plist already in place; also clears a disabled flag) or `lctl install <file>` (place + load) |
| `launchctl load -w <plist>` | `launchctl enable <target>` + `launchctl bootstrap <domain> <plist>` | `lctl enable <name>` |
| `launchctl unload <plist>` | `launchctl bootout <target>` | `lctl stop <name>` |
| `launchctl unload -w <plist>` | `launchctl disable <target>` + `launchctl bootout <target>` | `lctl disable <name>` |
| `launchctl remove <label>` | `launchctl bootout <target>` | `lctl stop <name>` |
| `launchctl start <label>` | `launchctl kickstart <target>` | `lctl start <name>` |
| `launchctl stop <label>` | `launchctl kill SIGTERM <target>` | `lctl stop <name>` (unloads instead, so `KeepAlive` can't restart it) |
| `launchctl list` | `launchctl print <domain>` | `lctl list` |
| `launchctl list <label>` | `launchctl print <target>` | `lctl status <name>` |
| `launchctl bootstrap <domain> <plist>` | — | `lctl enable <name>`, or `lctl start <name>` to also run it |
| `launchctl bootout <target>` | — | `lctl stop <name>` |
| `launchctl kickstart <target>` | — | `lctl start <name>` |
| `launchctl kickstart -k <target>` | — | `lctl restart <name>` |
| `launchctl enable <target>` | — | `lctl enable <name>` (also loads) |
| `launchctl disable <target>` | — | `lctl disable <name>` (also unloads) |
| `launchctl print <target>` | — | `lctl status -d <name>` |
| `launchctl kill <sig> <target>` | — | not wrapped; use launchctl |
| `launchctl print-disabled <domain>` | — | not wrapped; use launchctl |
| `launchctl blame <target>` | — | not wrapped; use launchctl |
| `launchctl submit`, `setenv`, `getenv`, `asuser`, `bsexec`, `limit`, `config`, `attach`, `debug`, `procinfo`, `dumpstate`, … | — | not wrapped; use launchctl |

The legacy subcommands (`load`, `unload`, `start`, `stop`, `list`, `remove`) infer the domain from who runs them: as root they target `system`, otherwise the current user's domain. lctl always passes an explicit domain.

### What lctl does around those commands

- **Discovery.** Every command except `completion` first runs `launchctl print system`, `launchctl print user/<uid>` and `launchctl print gui/<uid>`, and reads the plist directories listed under [Configuration](#configuration). A service listed by one of those `print` calls is loaded; one with PID `-` or `0` is not running.
- **Name resolution.** `<name>` is a case-insensitive substring of the label (`-e` for an exact match). If several services match, lctl shows a picker (`-y` picks the first alphabetically).
- **Domain resolution.** A service's domain is, in order of precedence: the domain where launchd currently has it loaded; the plist's `LimitLoadToSessionType` (`Aqua` → `gui`, `Background`/`LoginWindow` → `user`, `System` → `system`); the domain of the directory the plist was found in.
- **sudo.** lctl runs launchctl as you first. It retries as root only if launchctl's output contains `Try re-running the command as root`, or `Operation not permitted` without `System Integrity Protection` (sudo can't override SIP). With stdin on a terminal, lctl runs `/usr/bin/sudo -v`, which prompts the way sudo is configured (password, or Touch ID via `pam_tid`). It then reruns the command as `/usr/bin/sudo -n /bin/launchctl …`. Without a terminal there is no prompt and launchctl's error is reported as is. `install --domain system` and a cancelled prompt fail with "Sudo is required". lctl never reads or stores your password. launchctl prints the "re-running as root" hint on many failed `bootstrap` calls (for example, a disabled service), so such a failure can bring up a sudo prompt and then fail again with a more detailed error.
- **Running lctl under `sudo`** also works: it uses `SUDO_UID` to find your home and `gui/<uid>`.

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
