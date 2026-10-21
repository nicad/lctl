use assert_cmd::Command;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::fixtures::{self, PlistSpec, ProgramKind, Session};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub const FAIL_BUDGET: Duration = Duration::from_secs(1);
const POLL: Duration = Duration::from_millis(20);

pub fn uid() -> u32 {
    let out = StdCommand::new("id").arg("-u").output().expect("id -u");
    String::from_utf8_lossy(&out.stdout).trim().parse().expect("parse uid")
}

pub fn unique_label(cmd: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("com.lctl.test.{cmd}.{}.{}", std::process::id(), n)
}

pub fn lctl() -> Command {
    let mut cmd = Command::cargo_bin("lctl").expect("lctl binary");
    cmd.arg("-y");
    cmd
}

pub fn lctl_sudo() -> Command {
    let bin = assert_cmd::cargo::cargo_bin("lctl");
    let mut cmd = Command::new("sudo");
    // -n: non-interactive; warm ticket with `sudo -v` before `cargo e2e-all`.
    cmd.arg("-n").arg(bin).arg("-y");
    cmd
}

/// System e2e needs a cached sudo ticket. Prompting inside cargo test is unreliable
/// (parallel output buries `Password:`). Run `sudo -v && cargo e2e-all` instead.
pub fn require_root_e2e() {
    require_macos();
    assert!(
        sudo_n_ok(),
        "system e2e needs sudo without a prompt.\n\
         Run:  sudo -v && cargo e2e-all\n\
         (do not use `sudo cargo test` — only the ticket must be root-capable)"
    );
}

/// Interactive auto-elevate e2e: needs a controlling TTY so lctl's `stdin_is_tty()` is true
/// and `sudo -v` can prompt on `/dev/tty`.
pub fn require_interactive_sudo_e2e() {
    require_macos();
    File::open("/dev/tty").unwrap_or_else(|e| {
        panic!(
            "interactive sudo e2e needs a controlling TTY ({e}).\n\
             Run from a real terminal: sudo -v && cargo e2e-all"
        )
    });
}

pub fn sudo_n_ok() -> bool {
    StdCommand::new("sudo").args(["-n", "true"]).status().map(|s| s.success()).unwrap_or(false)
}

/// Drop the cached sudo timestamp (`sudo -K`).
pub fn invalidate_sudo_ticket() {
    let status = StdCommand::new("sudo").arg("-K").status().expect("spawn sudo -K");
    assert!(status.success(), "sudo -K failed");
}

/// Run `lctl` as the current user with stdin attached to `/dev/tty` (not under `sudo -n`).
/// stdout/stderr inherit so the sudo password prompt is visible.
pub fn lctl_tty_status(args: &[&str]) -> ExitStatus {
    let bin = assert_cmd::cargo::cargo_bin("lctl");
    let tty = File::open("/dev/tty").expect("open /dev/tty for interactive lctl");
    StdCommand::new(bin)
        .arg("-y")
        .args(args)
        .stdin(Stdio::from(tty))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("spawn lctl with /dev/tty stdin")
}

pub fn gui_target(label: &str) -> String {
    format!("gui/{}/{}", uid(), label)
}

pub fn system_target(label: &str) -> String {
    format!("system/{}", label)
}

pub fn agents_dir() -> PathBuf {
    dirs_home().join("Library/LaunchAgents")
}

pub fn daemons_dir() -> PathBuf {
    PathBuf::from("/Library/LaunchDaemons")
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).expect("HOME must be set for e2e tests")
}

pub fn installed_plist_path(label: &str, system: bool) -> PathBuf {
    let dir = if system { daemons_dir() } else { agents_dir() };
    dir.join(format!("{label}.plist"))
}

pub fn launchctl(args: &[&str]) -> std::process::Output {
    StdCommand::new("launchctl").args(args).output().expect("spawn launchctl")
}

pub fn launchctl_sudo(args: &[&str]) -> std::process::Output {
    let mut cmd = StdCommand::new("sudo");
    cmd.arg("-n").arg("launchctl").args(args);
    cmd.output().expect("spawn sudo launchctl")
}

pub fn print_service(target: &str) -> Option<String> {
    let out = launchctl(&["print", target]);
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if stdout.starts_with("Bad request.") {
        return None;
    }
    Some(stdout)
}

pub fn print_service_sudo(target: &str) -> Option<String> {
    let out = launchctl_sudo(&["print", target]);
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if stdout.starts_with("Bad request.") {
        return None;
    }
    Some(stdout)
}

pub fn is_loaded(target: &str) -> bool {
    print_service(target).is_some()
}

pub fn is_loaded_sudo(target: &str) -> bool {
    print_service_sudo(target).is_some()
}

pub fn pid_from_print(stdout: &str) -> Option<u32> {
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pid = ") {
            let pid = rest.trim();
            if pid != "0" {
                return pid.parse().ok();
            }
        }
    }
    None
}

pub fn is_running(target: &str) -> bool {
    print_service(target).and_then(|s| pid_from_print(&s)).is_some()
}

pub fn is_running_sudo(target: &str) -> bool {
    print_service_sudo(target).and_then(|s| pid_from_print(&s)).is_some()
}

/// Whether `launchctl print gui/$UID` lists the label as disabled.
/// Returns false if the domain cannot be printed (do not treat print failure as enabled).
pub fn is_disabled_in_domain(label: &str) -> bool {
    let domain = format!("gui/{}", uid());
    let Some(stdout) = print_service(&domain) else {
        return false;
    };
    disabled_map_value(&stdout, label) == Some("disabled")
}

/// Enabled = domain print succeeded and label is not mapped to `disabled`.
/// Print failure returns false so `assert_enabled` cannot false-pass.
pub fn is_enabled_in_domain(label: &str) -> bool {
    let domain = format!("gui/{}", uid());
    let Some(stdout) = print_service(&domain) else {
        return false;
    };
    disabled_map_value(&stdout, label) != Some("disabled")
}

pub fn is_disabled_in_system_domain(label: &str) -> bool {
    let Some(stdout) = print_service_sudo("system") else {
        return false;
    };
    disabled_map_value(&stdout, label) == Some("disabled")
}

pub fn is_enabled_in_system_domain(label: &str) -> bool {
    let Some(stdout) = print_service_sudo("system") else {
        return false;
    };
    disabled_map_value(&stdout, label) != Some("disabled")
}

fn disabled_map_value<'a>(domain_print: &'a str, label: &str) -> Option<&'a str> {
    let needle = format!("\"{label}\" => ");
    for line in domain_print.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(&needle) {
            return Some(rest.trim());
        }
    }
    None
}

pub fn assert_disabled(label: &str) {
    assert!(
        wait_until(|| is_disabled_in_domain(label), FAIL_BUDGET),
        "expected disabled in domain: {label}"
    );
}

pub fn assert_enabled(label: &str) {
    assert!(
        wait_until(|| is_enabled_in_domain(label), FAIL_BUDGET),
        "expected enabled in domain: {label}"
    );
}

pub fn assert_disabled_system(label: &str) {
    assert!(
        wait_until(|| is_disabled_in_system_domain(label), FAIL_BUDGET),
        "expected disabled in system domain: {label}"
    );
}

pub fn assert_enabled_system(label: &str) {
    assert!(
        wait_until(|| is_enabled_in_system_domain(label), FAIL_BUDGET),
        "expected enabled in system domain: {label}"
    );
}

pub fn wait_until(mut pred: impl FnMut() -> bool, budget: Duration) -> bool {
    let start = Instant::now();
    loop {
        if pred() {
            return true;
        }
        if start.elapsed() >= budget {
            return pred();
        }
        std::thread::sleep(POLL);
    }
}

pub fn assert_loaded(target: &str) {
    assert!(wait_until(|| is_loaded(target), FAIL_BUDGET), "expected loaded: {target}");
}

pub fn assert_not_loaded(target: &str) {
    assert!(wait_until(|| !is_loaded(target), FAIL_BUDGET), "expected not loaded: {target}");
}

pub fn assert_running(target: &str) {
    assert!(wait_until(|| is_running(target), FAIL_BUDGET), "expected running: {target}");
}

pub fn assert_not_running(target: &str) {
    assert!(wait_until(|| !is_running(target), FAIL_BUDGET), "expected not running: {target}");
}

pub fn assert_loaded_sudo(target: &str) {
    assert!(wait_until(|| is_loaded_sudo(target), FAIL_BUDGET), "expected loaded (sudo): {target}");
}

pub fn assert_not_loaded_sudo(target: &str) {
    assert!(
        wait_until(|| !is_loaded_sudo(target), FAIL_BUDGET),
        "expected not loaded (sudo): {target}"
    );
}

pub fn assert_running_sudo(target: &str) {
    assert!(
        wait_until(|| is_running_sudo(target), FAIL_BUDGET),
        "expected running (sudo): {target}"
    );
}

pub fn assert_not_running_sudo(target: &str) {
    assert!(
        wait_until(|| !is_running_sudo(target), FAIL_BUDGET),
        "expected not running (sudo): {target}"
    );
}

/// Test-owned launchd service. Cleanup uses raw launchctl only (never lctl).
pub struct TempService {
    pub label: String,
    pub target: String,
    pub fixture_plist: PathBuf,
    pub installed_plist: PathBuf,
    /// Kept alive so fixture files remain for the test duration.
    #[allow(dead_code)]
    pub temp: TempDir,
    pub system: bool,
    pub disabled: bool,
    pub stdout_log: Option<PathBuf>,
    #[allow(dead_code)]
    pub stderr_log: Option<PathBuf>,
}

pub struct TempServiceBuilder {
    cmd: String,
    system: bool,
    program: ProgramKind,
    run_at_load: bool,
    with_logs: bool,
}

impl TempServiceBuilder {
    pub fn gui(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            system: false,
            program: ProgramKind::True,
            run_at_load: false,
            with_logs: false,
        }
    }

    pub fn system(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            system: true,
            program: ProgramKind::True,
            run_at_load: false,
            with_logs: false,
        }
    }

    pub fn sleep(mut self) -> Self {
        self.program = ProgramKind::Sleep;
        self
    }

    pub fn echo_then_sleep(mut self) -> Self {
        self.program = ProgramKind::EchoThenSleep;
        self.with_logs = true;
        self
    }

    pub fn run_at_load(mut self) -> Self {
        self.run_at_load = true;
        self
    }

    pub fn build(self) -> TempService {
        let label = unique_label(&self.cmd);
        let temp = TempDir::new().expect("tempdir");
        let stdout_log = if self.with_logs { Some(temp.path().join("stdout.log")) } else { None };
        let stderr_log = if self.with_logs { Some(temp.path().join("stderr.log")) } else { None };
        if let Some(ref p) = stdout_log {
            std::fs::File::create(p).expect("touch stdout log");
        }
        if let Some(ref p) = stderr_log {
            std::fs::File::create(p).expect("touch stderr log");
        }

        let session = if self.system { Session::System } else { Session::Aqua };
        let fixture_plist = fixtures::write_plist(
            temp.path(),
            &PlistSpec {
                label: &label,
                session,
                program: self.program,
                run_at_load: self.run_at_load,
                stdout_path: stdout_log.as_deref(),
                stderr_path: stderr_log.as_deref(),
            },
        );

        let installed_plist = installed_plist_path(&label, self.system);
        let target = if self.system { system_target(&label) } else { gui_target(&label) };

        TempService {
            label,
            target,
            fixture_plist,
            installed_plist,
            temp,
            system: self.system,
            disabled: false,
            stdout_log,
            stderr_log,
        }
    }
}

impl TempService {
    pub fn install_via_lctl(&self) {
        let domain = if self.system { "system" } else { "gui" };
        let mut cmd = if self.system { lctl_sudo() } else { lctl() };
        // Default strategy is symlink (omit --strategy).
        cmd.args(["install", "--domain", domain, self.fixture_plist.to_str().unwrap()])
            .assert()
            .success();
        assert_installed_symlink(self);
        if self.system {
            assert_system_install_permissions(self);
            assert_loaded_sudo(&self.target);
        } else {
            assert_loaded(&self.target);
        }
    }

    /// Rewrite fixture plist in place (sudo when system — fixture may be root-owned).
    pub fn rewrite_fixture(&self, program: ProgramKind, run_at_load: bool) {
        let session = if self.system { Session::System } else { Session::Aqua };
        let xml = fixtures::render_plist_for_test(&PlistSpec {
            label: &self.label,
            session,
            program,
            run_at_load,
            stdout_path: self.stdout_log.as_deref(),
            stderr_path: self.stderr_log.as_deref(),
        });
        if self.system {
            write_root_file(&self.fixture_plist, &xml);
        } else {
            std::fs::write(&self.fixture_plist, xml).expect("rewrite fixture");
        }
    }

    /// Copy plist into LaunchAgents without loading it.
    pub fn place_gui_plist_unloaded(&self) {
        assert!(!self.system, "place_gui_plist_unloaded is gui-only");
        std::fs::create_dir_all(agents_dir()).ok();
        std::fs::copy(&self.fixture_plist, &self.installed_plist).expect("place plist");
        let _ = launchctl(&["bootout", &self.target]);
    }

    /// Place plist into LaunchDaemons as root-owned symlink (not loaded).
    pub fn place_system_plist_unloaded(&self) {
        assert!(self.system, "place_system_plist_unloaded is system-only");
        let src = self.fixture_plist.to_str().unwrap();
        let dest = self.installed_plist.to_str().unwrap();
        let status =
            StdCommand::new("sudo").args(["-n", "ln", "-sf", src, dest]).status().expect("sudo ln");
        assert!(status.success(), "sudo ln -sf failed");
        let status = StdCommand::new("sudo")
            .args(["-n", "chown", "-h", "root:wheel", dest])
            .status()
            .expect("sudo chown symlink");
        assert!(status.success(), "sudo chown -h root:wheel symlink failed");
        let status = StdCommand::new("sudo")
            .args(["-n", "chown", "root:wheel", src])
            .status()
            .expect("sudo chown fixture");
        assert!(status.success(), "sudo chown root:wheel fixture failed");
        let status = StdCommand::new("sudo")
            .args(["-n", "chmod", "644", src])
            .status()
            .expect("sudo chmod fixture");
        assert!(status.success(), "sudo chmod 644 fixture failed");
        let _ = launchctl_sudo(&["bootout", &self.target]);
    }

    /// Place plist in LaunchAgents/Daemons and bootstrap with raw launchctl (no lctl).
    #[allow(dead_code)]
    pub fn install_raw(&self) {
        let dest = &self.installed_plist;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if self.system {
            let status = StdCommand::new("sudo")
                .args(["-n", "cp", self.fixture_plist.to_str().unwrap(), dest.to_str().unwrap()])
                .status()
                .expect("sudo cp");
            assert!(status.success(), "sudo cp plist failed");
            let out = launchctl_sudo(&["bootstrap", "system", dest.to_str().unwrap()]);
            assert!(
                out.status.success(),
                "sudo bootstrap failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert_loaded_sudo(&self.target);
        } else {
            std::fs::copy(&self.fixture_plist, dest).expect("copy plist to LaunchAgents");
            let domain = format!("gui/{}", uid());
            let out = launchctl(&["bootstrap", &domain, dest.to_str().unwrap()]);
            assert!(
                out.status.success(),
                "bootstrap failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert_loaded(&self.target);
        }
    }

    pub fn mark_disabled(&mut self) {
        self.disabled = true;
    }

    fn reclaim_temp_tree(&self) {
        if !self.system {
            return;
        }
        // System symlink install chowns the fixture to root; reclaim so TempDir can remove it.
        let Some(path) = self.temp.path().to_str() else {
            return;
        };
        let _ =
            StdCommand::new("sudo").args(["-n", "chown", "-R", &uid().to_string(), path]).status();
    }

    pub fn cleanup(&mut self) {
        if self.system {
            let _ = launchctl_sudo(&["bootout", &self.target]);
            if self.disabled {
                let _ = launchctl_sudo(&["enable", &self.target]);
            }
            if self.installed_plist.exists()
                || std::fs::symlink_metadata(&self.installed_plist).is_ok()
            {
                let _ = StdCommand::new("sudo")
                    .args(["-n", "rm", "-f", self.installed_plist.to_str().unwrap()])
                    .status();
            }
            self.reclaim_temp_tree();
        } else {
            let _ = launchctl(&["bootout", &self.target]);
            if self.disabled {
                let _ = launchctl(&["enable", &self.target]);
            }
            let _ = std::fs::remove_file(&self.installed_plist);
        }
    }
}

impl Drop for TempService {
    fn drop(&mut self) {
        self.cleanup();
    }
}

pub fn require_macos() {
    #[cfg(not(target_os = "macos"))]
    panic!("e2e tests require macOS");
}

/// Default install strategy: LaunchAgents/Daemons entry is a symlink to the fixture.
pub fn assert_installed_symlink(svc: &TempService) {
    let meta = std::fs::symlink_metadata(&svc.installed_plist).unwrap_or_else(|e| {
        panic!("installed plist metadata {}: {e}", svc.installed_plist.display())
    });
    assert!(meta.file_type().is_symlink(), "expected symlink at {}", svc.installed_plist.display());
    let link = std::fs::read_link(&svc.installed_plist).expect("read_link installed plist");
    let link_canon = std::fs::canonicalize(&link)
        .unwrap_or_else(|e| panic!("canonicalize symlink target {}: {e}", link.display()));
    let fixture_canon = std::fs::canonicalize(&svc.fixture_plist).expect("canonicalize fixture");
    assert_eq!(link_canon, fixture_canon, "installed symlink should point at fixture plist");
}

/// System symlink install: root:wheel on symlink + fixture, fixture mode 644.
pub fn assert_system_install_permissions(svc: &TempService) {
    assert!(svc.system, "assert_system_install_permissions is system-only");

    let link_meta = std::fs::symlink_metadata(&svc.installed_plist).expect("symlink metadata");
    assert!(link_meta.file_type().is_symlink(), "LaunchDaemons entry should be a symlink");
    assert_eq!(link_meta.uid(), 0, "symlink uid should be root (0)");
    assert_eq!(link_meta.gid(), 0, "symlink gid should be wheel (0)");

    let fixture_meta = std::fs::metadata(&svc.fixture_plist).expect("fixture metadata");
    assert_eq!(fixture_meta.uid(), 0, "fixture uid should be root (0)");
    assert_eq!(fixture_meta.gid(), 0, "fixture gid should be wheel (0)");
    assert_eq!(
        fixture_meta.mode() & 0o777,
        0o644,
        "fixture mode should be 644, got {:o}",
        fixture_meta.mode() & 0o777
    );

    // Follow symlink: same root-owned 644 target launchd will read.
    let followed = std::fs::metadata(&svc.installed_plist).expect("followed installed metadata");
    assert_eq!(followed.uid(), 0, "followed plist uid should be root");
    assert_eq!(followed.gid(), 0, "followed plist gid should be wheel");
    assert_eq!(followed.mode() & 0o777, 0o644, "followed plist mode should be 644");
}

fn write_root_file(path: &Path, contents: &str) {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = StdCommand::new("sudo")
        .args(["-n", "tee", path.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("sudo tee");
    child
        .stdin
        .as_mut()
        .expect("tee stdin")
        .write_all(contents.as_bytes())
        .expect("write tee stdin");
    let status = child.wait().expect("wait tee");
    assert!(status.success(), "sudo tee failed for {}", path.display());
    let status = StdCommand::new("sudo")
        .args(["-n", "chown", "root:wheel", path.to_str().unwrap()])
        .status()
        .expect("sudo chown after tee");
    assert!(status.success(), "sudo chown after tee failed");
    let status = StdCommand::new("sudo")
        .args(["-n", "chmod", "644", path.to_str().unwrap()])
        .status()
        .expect("sudo chmod after tee");
    assert!(status.success(), "sudo chmod after tee failed");
}

#[allow(dead_code)]
pub fn fixture_path(svc: &TempService) -> &Path {
    &svc.fixture_plist
}
