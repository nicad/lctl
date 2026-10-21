# Interactive discovery: deferred items

Status: planned, not started.

The goal is progressive discovery: using launchd services without knowing labels, domains or launchctl verbs up front. Ideas partly come from [launchy](https://github.com/dalberto/launchy).

Already in scope (being built separately): the overview for `lctl` / `lctl status` with no argument and the new `list` columns and filters, the one-screen `status <name>` summary, `lctl doctor`, and error messages that name the fix. Everything below is deferred.

## 1. Picker when the service name is omitted

`lctl start` with no name opens a fuzzy-searchable list of candidates instead of failing:

| Command | Candidates |
|---|---|
| `start` | services that are not running |
| `stop`, `restart` | running services |
| `enable` | disabled services |
| `disable` | services that are not disabled |
| `status`, `logs`, `plist`, `reload`, `uninstall`, `doctor` | all of your services |

Notes:
- `status` and `doctor` with no name already mean "overview" / "check everything". Decide whether the picker replaces that or needs a flag (e.g. `-p`).
- dialoguer, already a dependency, has `FuzzySelect` behind its `fuzzy-select` feature (pulls in `fuzzy-matcher`).
- Without a terminal, keep failing with "missing name" as today. `-y` should not pick for you.
- Reuse the list filters (state, Apple, plist on disk) to build the candidates.

## 2. Tab completion of service labels

`lctl start zo<TAB>` → `us.zoom.ZoomDaemon`.

- clap_complete's dynamic completion (`CompleteEnv`, `unstable-dynamic` feature) calls the binary at completion time. Users set it up with e.g. `source <(COMPLETE=zsh lctl)` instead of the static script from `lctl completion`.
- Each TAB runs discovery (three `launchctl print` calls plus plist directory reads). Measure the latency; if it's noticeable, cache labels for a few seconds.
- Complete only the labels that make sense per command, as for the picker.
- zsh and bash filter candidates by prefix; substring matching needs `fzf-tab` or similar. Document this.

## 3. Smaller additions

- **`logs -f` and `-n <lines>`**: follow the stdout/stderr files (`tail -F` behaviour) together with `log stream --predicate 'process == "<program>"'`. `-n` replaces the fixed 50 lines.
- **`--json`** for `list`, `status` and `doctor`. Sketch: `{label, domain, state, pid, last_exit_code, disabled, plist, runs_when, program, arguments, checks: [{severity, message, fix}]}`.
- **Create a plist from flags**: `lctl install <label> --program … [--program …] --every 5m | --at 02:00 | --calendar day=1,hour=12 --env K=V --log-dir DIR --keep-alive --run-at-load --watch PATH --queue-dir DIR --start-on-mount`, as launchy does. Write the plist to `~/Library/Application Support/lctl/<domain>/<label>.plist`, install it with the existing symlink flow, and run doctor on the result.
- **Bulk operations** (`--prefix`, `--grep` on start/stop/enable/disable/uninstall with a count + confirmation): low value for discovery. Skip unless a need shows up.

## Other follow-ups

- `uninstall` deletes the plist without sudo, so uninstalling a system service needs `sudo lctl uninstall`. Use `sudo -n /bin/rm` like `install` does.
- `Daemon` keeps `loaded`, `pid` and `last_exit_code` as separate fields that must agree. Consider one `Option<LaunchdState>` set by discovery.
