use crate::config::Config;
use crate::error::{Error, Result};
use crate::output::stderr;
use crate::parser;
use crate::types::{Daemon, Domain, ForWhom, Plist};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

struct PlistDir {
    path: PathBuf,
    domain: Domain,
    for_use_by: ForWhom,
    is_system_monitored: bool,
}

fn get_plist_dirs(
    config: &Config,
    user_uid: u32,
    lctl_dir: &std::path::Path,
    home: &std::path::Path,
) -> Vec<PlistDir> {
    let mut dirs = vec![
        PlistDir {
            path: PathBuf::from("/System/Library/LaunchDaemons"),
            domain: Domain::System,
            for_use_by: ForWhom::Apple,
            is_system_monitored: true,
        },
        PlistDir {
            path: PathBuf::from("/System/Library/LaunchAgents"),
            domain: Domain::Gui(user_uid),
            for_use_by: ForWhom::Apple,
            is_system_monitored: true,
        },
        PlistDir {
            path: PathBuf::from("/System/Library/LaunchAngels"),
            domain: Domain::Gui(user_uid),
            for_use_by: ForWhom::Apple,
            is_system_monitored: true,
        },
        PlistDir {
            path: PathBuf::from("/Library/LaunchDaemons"),
            domain: Domain::System,
            for_use_by: ForWhom::ThirdParty,
            is_system_monitored: true,
        },
        PlistDir {
            path: PathBuf::from("/Library/LaunchAgents"),
            domain: Domain::Gui(user_uid),
            for_use_by: ForWhom::ThirdParty,
            is_system_monitored: true,
        },
    ];

    dirs.push(PlistDir {
        path: home.join("Library/LaunchAgents"),
        domain: Domain::Gui(user_uid),
        for_use_by: ForWhom::User,
        is_system_monitored: true,
    });

    for (path, domain, for_use_by) in crate::lctl::lctl_domain_dirs(lctl_dir, user_uid) {
        dirs.push(PlistDir { path, domain, for_use_by, is_system_monitored: false });
    }

    // Add user-defined directories from config
    for cfg_dir in config.directories() {
        dirs.push(PlistDir {
            path: cfg_dir.path.clone(),
            domain: cfg_dir.domain.to_domain(user_uid),
            for_use_by: ForWhom::User,
            is_system_monitored: false,
        });
    }

    dirs
}

pub struct DaemonRegistry {
    map: HashMap<String, Daemon>,
}

impl DaemonRegistry {
    pub fn new(
        config: &Config,
        uid: u32,
        lctl_dir: &std::path::Path,
        home: &std::path::Path,
    ) -> Result<Self> {
        let plist_dirs = get_plist_dirs(config, uid, lctl_dir, home);
        let mut map: HashMap<String, Daemon> = HashMap::new();

        // Scan plist directories
        for plist_dir in &plist_dirs {
            let Ok(entries) = std::fs::read_dir(&plist_dir.path) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("plist") {
                    continue;
                }
                let plist_uid = match plist_dir.domain {
                    Domain::User(uid) | Domain::Gui(uid) => uid,
                    _ => 0,
                };
                if let Ok(content) = std::fs::read(&path) {
                    if let Ok(plist_data) = plist::from_bytes::<Plist>(&content) {
                        // pick source path for duplicates
                        let label = plist_data.label.as_deref().unwrap_or("");
                        let found_path =
                            map.get(label).and_then(|daemon| daemon.source_path.clone());
                        let source_path = match found_path {
                            Some(_) if path.is_symlink() => path,
                            Some(found_path) if found_path.is_symlink() => found_path,
                            Some(found_path) if plist_dir.is_system_monitored => found_path,
                            Some(_) => path,
                            None => path,
                        };

                        let daemon = Daemon::from_plist(
                            plist_data,
                            Some(source_path),
                            plist_dir.domain,
                            plist_dir.for_use_by,
                            plist_uid,
                        );
                        map.insert(daemon.name.clone(), daemon);
                    }
                }
            }
        }

        // Add running daemons from launchctl print (parallelized)
        let domains = vec![Domain::System, Domain::User(uid), Domain::Gui(uid)];
        let handles: Vec<_> = domains
            .into_iter()
            .map(|domain| {
                thread::spawn(move || match parser::parse_print_domain(&domain) {
                    Ok(services) => Some((domain, services)),
                    Err(e) => {
                        stderr::error(&format!("Failed to parse domain {}: {}", domain, e));
                        None
                    }
                })
            })
            .collect();

        for handle in handles {
            let joined =
                handle.join().map_err(|_| Error::unexpected("domain scan thread panicked"))?;
            if let Some((domain, services)) = joined {
                for service in services {
                    // Update existing daemon or create new one
                    if let Some(daemon) = map.get_mut(&service.name) {
                        daemon.domain = domain;
                        daemon.pid = service.pid;
                        daemon.last_exit_code = service.last_exit_code;
                        daemon.loaded = true;
                    } else {
                        let for_use_by = if service.name.contains("com.apple") {
                            ForWhom::Apple
                        } else {
                            ForWhom::ThirdParty
                        };
                        let name = service.name.clone();
                        let daemon = Daemon {
                            name: service.name,
                            source_path: None,
                            domain,
                            for_use_by,
                            plist: None,
                            pid: service.pid,
                            last_exit_code: service.last_exit_code,
                            loaded: true,
                        };
                        map.insert(name, daemon);
                    }
                }
            }
        }

        Ok(Self { map })
    }

    pub fn get_matches(&self, query: &str, exact: bool) -> Vec<&Daemon> {
        if exact {
            return self
                .map
                .values()
                .filter(|daemon| daemon.name.eq_ignore_ascii_case(query))
                .collect();
        }
        let query_lower = query.to_ascii_lowercase();
        self.map
            .values()
            .filter(|daemon| daemon.name.to_ascii_lowercase().contains(&query_lower))
            .collect()
    }

    /// Fuzzy matches restricted to plists loaded from lctl-managed dirs.
    pub fn get_lctl_matches(&self, query: &str) -> Vec<&Daemon> {
        self.get_matches(query, false)
            .into_iter()
            .filter(|d| d.for_use_by == ForWhom::Lctl)
            .collect()
    }
}
