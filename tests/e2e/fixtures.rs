use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub enum Session {
    Aqua,
    System,
}

#[derive(Clone, Copy)]
pub enum ProgramKind {
    /// `/usr/bin/true`, does not stay running
    True,
    /// `/bin/sleep 999999`, stays running after kickstart / RunAtLoad
    Sleep,
    /// Writes a marker line to stdout then sleeps (for logs tests)
    EchoThenSleep,
}

pub struct PlistSpec<'a> {
    pub label: &'a str,
    pub session: Session,
    pub program: ProgramKind,
    pub run_at_load: bool,
    pub stdout_path: Option<&'a Path>,
    pub stderr_path: Option<&'a Path>,
}

pub fn write_plist(dir: &Path, spec: &PlistSpec<'_>) -> PathBuf {
    let path = dir.join(format!("{}.plist", spec.label));
    let xml = render_plist(spec);
    fs::write(&path, xml).expect("write fixture plist");
    path
}

/// Public for e2e rewrite helpers (system reboot etc.).
pub fn render_plist_for_test(spec: &PlistSpec<'_>) -> String {
    render_plist(spec)
}

fn render_plist(spec: &PlistSpec<'_>) -> String {
    let session = match spec.session {
        Session::Aqua => "Aqua",
        Session::System => "System",
    };
    let run_at_load = if spec.run_at_load { "true" } else { "false" };

    let (program_xml, args_xml) = match spec.program {
        ProgramKind::True => (
            "\t<key>Program</key>\n\t<string>/usr/bin/true</string>\n".to_string(),
            String::new(),
        ),
        ProgramKind::Sleep => (
            String::new(),
            "\t<key>ProgramArguments</key>\n\t<array>\n\t\t<string>/bin/sleep</string>\n\t\t<string>999999</string>\n\t</array>\n"
                .to_string(),
        ),
        ProgramKind::EchoThenSleep => (
            String::new(),
            "\t<key>ProgramArguments</key>\n\t<array>\n\t\t<string>/bin/sh</string>\n\t\t<string>-c</string>\n\t\t<string>echo angel-e2e-marker; exec /bin/sleep 999999</string>\n\t</array>\n"
                .to_string(),
        ),
    };

    let mut stdio = String::new();
    if let Some(p) = spec.stdout_path {
        stdio.push_str(&format!(
            "\t<key>StandardOutPath</key>\n\t<string>{}</string>\n",
            p.display()
        ));
    }
    if let Some(p) = spec.stderr_path {
        stdio.push_str(&format!(
            "\t<key>StandardErrorPath</key>\n\t<string>{}</string>\n",
            p.display()
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{label}</string>
	<key>LimitLoadToSessionType</key>
	<string>{session}</string>
	<key>RunAtLoad</key>
	<{run_at_load}/>
{program}{args}{stdio}</dict>
</plist>
"#,
        label = spec.label,
        session = session,
        run_at_load = run_at_load,
        program = program_xml,
        args = args_xml,
        stdio = stdio,
    )
}
