# lctl

A simpler CLI on top of macOS `launchctl`.

Fork of [angel](https://github.com/oliviaBahr/angel) by Olivia Bahr (MIT), renamed and maintained separately.

- `lctl` on its own lists your services with their state (running, idle, failed, not loaded, disabled), how long they have been up and when they run
- `lctl status <name>`: state, command, schedule, logs and plist on one screen, with problems and the next commands to run
- `lctl doctor` checks plists for problems launchd won't tell you about (missing program, unwritable log directory, wrong owner, edits not reloaded…)
- Commands named after what you want (`start`, `stop`, `enable`, `disable`, `reload`) that run the needed launchctl steps
- launchctl errors come with a hint naming the lctl command that fixes them
- Fuzzy (substring) service name matching, with a picker when several services match
- `install` / `uninstall` that place the plist and load it in one step
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
lctl                        # your services: state, uptime, schedule (lctl ls -l: domain, plist)
lctl status my.service      # one service: what it runs, when, logs, problems, next steps
lctl doctor                 # check all your services for problems
lctl explain stop           # the launchctl calls behind a command (also: lctl explain load)

lctl install ./my.service.plist
lctl start my.service
lctl status my.service
lctl logs my.service
lctl stop my.service
lctl uninstall my.service

lctl restart my.service
lctl reload my.service      # pick up plist edits
lctl kill -s HUP my.service # send a signal (default TERM); it stays loaded

lctl ls backup              # the services matching "backup"
lctl restart -A backup      # restart all of them (lists them and asks first)
lctl status -A backup       # status of each
lctl disable my.service     # stop it and keep it off across reboots
lctl enable my.service      # undo disable, and load it
lctl plist my.service       # print the plist file
```

Common flags:

- `-v` / `--verbose` — verbose output, including every command lctl executes
- `-y` / `--yes` — assume yes for confirmation prompts (and pick the first match when several services match)
- `-e` / `--exact` — exact (case-insensitive) service name match instead of substring
- `-A` / `--all` — act on every service that matches the name (what `lctl list <name>` shows) instead of picking one: `status`, `doctor`, `start`, `stop`, `restart`, `reload`, `kill`, `enable`, `disable`
- `install --domain gui|user|system` — skip interactive domain selection
- `install -s symlink|copy|move` — how to place the plist (default: symlink)
- `logs -t 5m` — log time frame (e.g. `1h`, `24h`)
- `kill -s HUP` — signal name (`TERM`, `HUP`, `KILL`…, with or without `SIG`) or number; default `TERM`
- `list --running` / `--idle` / `--failed` / `--not-loaded` / `--disabled` — only services in those states (combinable)
- `list -l` (also `status -l` without a name) — also show the Domain and Plist columns
- `list -a` / `-d` — also show Apple services / services without a plist on disk
- `list -1` — labels only, one per line (for scripts)
- `status -d` — also show every launchd field and the raw `launchctl print` output

Run `lctl --help` or `lctl <command> --help` for options, and `lctl explain <command>` for the launchctl calls behind a command. See `man lctl` for the full manual.

<!-- lctl explain: start -->
<!-- Compiled into the binary and printed by `lctl explain`, up to the end marker.
     Keep to headings, paragraphs, lists and tables (cells may use <br>). -->
## How lctl maps to launchctl

This section is also built into lctl: `lctl explain` prints it, and `lctl explain <command>` prints only what concerns one lctl or launchctl command (e.g. `lctl explain stop`, `lctl explain load`).

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
| `start <name>` | 1. if not loaded: `launchctl bootstrap <domain> <plist>`<br>2. `launchctl kickstart <target>` | `kickstart` runs the process now, whatever the plist's launch conditions. An already running process is left alone. A disabled service can't be loaded: if it is disabled and not loaded, lctl stops before calling launchctl and says to run `lctl enable`. |
| `stop <name>` | if loaded: `launchctl bootout <target>` | If it isn't loaded, nothing runs. Because the service is unloaded, `KeepAlive` can't restart it. It comes back at the next login/boot if the plist is in a standard Launch* directory; `disable` prevents that. |
| `restart <name>` | if loaded: `launchctl kickstart -k <target>`<br>if not loaded: 1. `launchctl bootstrap <domain> <plist>` 2. `launchctl kickstart <target>` | `-k` kills the running process first, if there is one. When not loaded it behaves like `start`, including the disabled check. |
| `reload <name>` | 1. if loaded: `launchctl bootout <target>`<br>2. `launchctl bootstrap <domain> <plist>` | Picks up plist edits and symlink target changes. Fails before unloading if the service has no plist on disk or is disabled. |
| `kill <name>` | `launchctl kill <signal> <target>` | `<signal>` comes from `-s` (default `TERM`): a name such as `TERM`, `HUP`, `INT` or `KILL`, with or without `SIG`, in any case, or a number. The service stays loaded, so launchd starts it again if its `KeepAlive` asks for that; `stop` unloads it instead. If it isn't running, lctl stops before calling launchctl. |
| `enable <name>` | 1. `launchctl enable <target>`<br>2. if not loaded: `launchctl bootstrap <domain> <plist>` | The flag change persists across reboots. Loading runs the process only if the plist asks for it; `start` runs it now. |
| `disable <name>` | 1. `launchctl disable <target>`<br>2. if loaded: `launchctl bootout <target>` | Persists across reboots until `enable`. Disabling before the bootout keeps `KeepAlive` from restarting it. The plist stays on disk. |
| `status <name>` | if loaded: `launchctl print <target>` | Shows the state, domain, enabled/disabled, PID, when the process started and its uptime, the command (from the plist, or from launchd when there is no plist), "runs when", log files, plist path, problems from the [doctor checks](#doctor-checks) and the next commands to run. "Next" lists the fixes for any problems; without problems, the usual commands for its state (running: `logs`, `restart`, `stop`; idle: `start`). `-d` adds every launchd field and the raw `launchctl print` output. |
| `status` | nothing beyond discovery | Same as `list` (`-l` too). |
| `lctl` (no command) | nothing beyond discovery | Same as `list`. |
| `list [pattern]` | nothing beyond discovery | Columns: state, PID, when the process started (the time if today, else the date) and its uptime, "runs when" (from the plist's `RunAtLoad`, `KeepAlive`, `StartInterval`, `StartCalendarInterval`, `WatchPaths`, `QueueDirectories`, `StartOnMount`, `Sockets`/`MachServices`; "manual only" when none), label; `-l` adds domain and plist path. Then a count per state. Shows services that have a plist on disk and aren't Apple's, in any state. `--running`, `--idle`, `--failed`, `--not-loaded`, `--disabled` keep only those states (combinable); `-a` adds Apple's, `-d` adds services without a plist; `-1` prints labels only. "Apple" means a plist under `/System/Library`, or, for a service without a plist, a label containing `com.apple`. |
| `doctor [name]` | for each loaded service: `launchctl print <target>` | Runs the [doctor checks](#doctor-checks). Without a name it checks all the services `list` shows by default and prints only those with problems (`-v`: all, including passing checks). Exits with an error if any check fails. |
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
6. **Existing service.** If a service with that label is loaded, lctl asks (default no), then runs `launchctl disable <target>`, `launchctl bootout <target>`, `launchctl enable <target>`. If it isn't loaded but is disabled, lctl runs `launchctl enable <target>`.
7. `launchctl bootstrap <domain> <plist>`
8. Prints any problems the [doctor checks](#doctor-checks) find in the installed service.

For `system` when lctl isn't running as root, steps 3–5 run through `sudo -n`: `/bin/rm -f`, `/bin/ln -sf`, `/bin/cp`, `/bin/mv`, `/usr/sbin/chown -h root:wheel`, `/bin/chmod 644`. As root, lctl makes the same changes directly.

#### uninstall

`lctl uninstall <name>`:

1. Asks for confirmation (default no; `-y` skips it).
2. `launchctl bootout <target>`. Best-effort: a failure (for example, not loaded) is ignored and shown only with `-v`.
3. Deletes `<plist>`: the plist found during discovery or, for a service without one, the `path` reported by `launchctl print <target>`. If it is a symlink, only the link is deleted, not the file it points to. This step does not use sudo, so for a plist in `/Library/LaunchDaemons` run `sudo lctl uninstall <name>`.
4. `launchctl enable <target>`. Best-effort: clears a disabled flag so a later install of the same label can load. A failure prints a warning.

#### doctor checks

Used by `lctl doctor`, `lctl status` and after `lctl install`. ✗ is a failure (launchd won't load or run it as intended), ! a warning; most come with the command that fixes them.

| Check | Result |
|---|---|
| State | ! failed (last exit code non-zero) → `lctl logs`; ! disabled → `lctl enable`; ! not loaded while the plist is in a Launch* directory → `lctl start`; ! not installed (plist outside the Launch* directories) → `lctl install` |
| `Label` | ✗ missing (e.g. an emptied stub plist): launchd ignores the file and nothing else is checked; ! file name isn't `<Label>.plist` |
| Plist owner and mode | ✗ if a daemon's plist, or any plist under `/Library`, isn't owned by root, or another agent plist isn't owned by you (or root), or the file is writable by group/others (checked on the symlink target for symlinks) → the `chown`/`chmod` to run |
| `LimitLoadToSessionType` | ! a value other than `System` in a LaunchDaemons directory, or `System` in a LaunchAgents directory |
| Program | ✗ no `Program`/`ProgramArguments`; ✗ an absolute path that doesn't exist or isn't executable; ✗ a bare name not found in the plist's `PATH` or launchd's default `/usr/bin:/bin:/usr/sbin:/sbin` |
| `WorkingDirectory` | ✗ doesn't exist |
| Log files | ! the directory of `StandardOutPath`/`StandardErrorPath` doesn't exist, so output is lost → `mkdir -p`, with `sudo` (and a `chown` to you for agents) when you can't write the parent; ! for agents, the log file or directory isn't writable by you |
| Launch triggers | ! none (only runs when started) → `lctl start`; ! `StartInterval` under 10 s |
| Loaded vs disk (loaded services) | ! program or arguments that launchd loaded differ from the plist on disk → `lctl reload`; ✗ loaded from a plist file that no longer exists → `lctl stop` |

### launchctl → lctl

| launchctl | Closest modern equivalent | lctl |
|---|---|---|
| `launchctl load <plist>` | `launchctl bootstrap <domain> <plist>` | `lctl enable <name>` (plist already in place; also clears a disabled flag) or `lctl install <file>` (place + load) |
| `launchctl load -w <plist>` | `launchctl enable <target>` + `launchctl bootstrap <domain> <plist>` | `lctl enable <name>` |
| `launchctl unload <plist>` | `launchctl bootout <target>` | `lctl stop <name>` |
| `launchctl unload -w <plist>` | `launchctl disable <target>` + `launchctl bootout <target>` | `lctl disable <name>` |
| `launchctl remove <label>` | `launchctl bootout <target>` | `lctl stop <name>` |
| `launchctl start <label>` | `launchctl kickstart <target>` | `lctl start <name>` |
| `launchctl stop <label>` | `launchctl kill SIGTERM <target>` | `lctl kill <name>` (same signal; the service stays loaded, so `KeepAlive` can restart it) or `lctl stop <name>` (unloads, so it can't) |
| `launchctl list` | `launchctl print <domain>` | `lctl` or `lctl list` |
| `launchctl list <label>` | `launchctl print <target>` | `lctl status <name>` |
| `launchctl bootstrap <domain> <plist>` | — | `lctl enable <name>`, or `lctl start <name>` to also run it |
| `launchctl bootout <target>` | — | `lctl stop <name>` |
| `launchctl kickstart <target>` | — | `lctl start <name>` |
| `launchctl kickstart -k <target>` | — | `lctl restart <name>` |
| `launchctl enable <target>` | — | `lctl enable <name>` (also loads) |
| `launchctl disable <target>` | — | `lctl disable <name>` (also unloads) |
| `launchctl print <target>` | — | `lctl status -d <name>` |
| `launchctl kill <sig> <target>` | — | `lctl kill -s <sig> <name>` |
| `launchctl print-disabled <domain>` | — | `lctl list --disabled` (`-a -d` to include Apple's and services without a plist; launchctl also lists flags left for labels that no longer exist) |
| `launchctl blame <target>` | — | not wrapped; use launchctl |
| `launchctl submit`, `setenv`, `getenv`, `asuser`, `bsexec`, `limit`, `config`, `attach`, `debug`, `procinfo`, `dumpstate`, … | — | not wrapped; use launchctl |

The legacy subcommands (`load`, `unload`, `start`, `stop`, `list`, `remove`) infer the domain from who runs them: as root they target `system`, otherwise the current user's domain. lctl always passes an explicit domain.

### What lctl does around those commands

- **Discovery.** Every command except `completion` first runs `launchctl print system`, `launchctl print user/<uid>` and `launchctl print gui/<uid>`, and reads the plist directories listed under [Configuration](#configuration). A service listed by one of those `print` calls is loaded; one with PID `-` or `0` is not running. The same output's `disabled services` block gives each label's disabled flag. The state shown everywhere is: running (has a PID), failed (loaded, last exit code non-zero), idle (loaded otherwise), disabled (not loaded, flag set), not loaded.
- **Start time and uptime.** launchd doesn't report when a process started. lctl asks the kernel for the PID's start time (`sysctl` `kern.proc.pid.<pid>`, which `ps` also reads); this works for root daemons without root and runs no extra program. Services that aren't running have none: launchd doesn't record when past runs started or ended.
- **Error hints.** When a launchctl call fails, lctl adds a `hint:` line based on launchd's message: `Could not find service` (113) → `lctl start`; `Input/output error` (5, launchd's catch-all for a refused load) → `lctl reload` if it is already loaded, otherwise `lctl doctor` (for the `user` domain, also a note that agents for your login session belong in `gui`; a disabled service is caught before launchctl runs); `Operation already in progress` (37) → `lctl reload`; `Service is disabled` (119) → `lctl enable`; `Path had bad ownership/permissions` (122) → `lctl doctor`; `Invalid signal` (64) → signal names to use; SIP (150) → can't be changed; `Operation not permitted` (1) or the "re-run as root" message with no more specific error → needs root.
- **Name resolution.** `<name>` is a case-insensitive substring of the label (`-e` for an exact match). If several services match, lctl shows a picker (`-y` picks the first alphabetically; without a terminal, lctl stops and lists the options).
- **Several services (`-A`).** With `-A`, the command acts on every service `lctl list <name>` shows (by default those with a plist on disk that aren't Apple's), in label order, running the same steps as for one service. Commands that change something first list the services and ask once (default no; `-y` skips; without a terminal, `-y` is required). A failure is printed and the other services still run; the command then fails with "N of M services failed". `doctor -A` reports like `doctor` without a name: only services with problems, then a count.
- **Domain resolution.** A service's domain is, in order of precedence: the domain where launchd currently has it loaded; the plist's `LimitLoadToSessionType` (`Aqua` → `gui`, `Background`/`LoginWindow` → `user`, `System` → `system`); the domain of the directory the plist was found in.
- **sudo.** lctl runs launchctl as you first. It retries as root only if launchctl's output contains `Try re-running the command as root`, or `Operation not permitted` without `System Integrity Protection` (sudo can't override SIP). With stdin on a terminal, lctl runs `/usr/bin/sudo -v`, which prompts the way sudo is configured (password, or Touch ID via `pam_tid`). It then reruns the command as `/usr/bin/sudo -n /bin/launchctl …`. If the prompt is declined, lctl doesn't ask again during the same run (for example for the other services of `-A`). Without a terminal there is no prompt and launchctl's error is reported as is. `install --domain system` and a cancelled prompt fail with "Sudo is required". lctl never reads or stores your password. launchctl prints the "re-running as root" hint on many failed `bootstrap` calls (for example, a disabled service), so such a failure can bring up a sudo prompt and then fail again with a more detailed error.
- **Running lctl under `sudo`** also works: it uses `SUDO_UID` to find your home and `gui/<uid>`.

<!-- lctl explain: end -->
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
