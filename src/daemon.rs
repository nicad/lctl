use crate::config::Config;
use crate::error::{Error, Result, err_msg};
use crate::output::stderr;
use crate::parser;
use crate::styles;
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

fn get_plist_dirs(config: &Config, user_uid: u32, angel_dir: &std::path::Path) -> Vec<PlistDir> {
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
            domain: Domain::User(user_uid),
            for_use_by: ForWhom::ThirdParty,
            is_system_monitored: true,
        },
    ];

    let home = crate::utils::home();
    dirs.push(PlistDir {
        path: home.join("Library/LaunchAgents"),
        domain: Domain::User(user_uid),
        for_use_by: ForWhom::User,
        is_system_monitored: true,
    });

    for (path, domain, for_use_by) in crate::angel::angel_domain_dirs(angel_dir, user_uid) {
        dirs.push(PlistDir { path, domain, for_use_by, is_system_monitored: false });
    }

    // Add user-defined directories from config
    for cfg_dir in config.get_directories() {
        dirs.push(PlistDir {
            path: PathBuf::from(&cfg_dir.path),
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
    pub fn new(config: &Config, uid: u32, angel_dir: &std::path::Path) -> Result<Self> {
        let plist_dirs = get_plist_dirs(config, uid, angel_dir);
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
                        let found_daemon = map.get(&plist_data.label.clone().unwrap_or_default());
                        let found_path = found_daemon.and_then(|daemon| daemon.source_path.clone());
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
            if let Some((domain, services)) = handle.join().unwrap() {
                for (pid, last_exit_code, name) in services {
                    // Update existing daemon or create new one
                    if let Some(daemon) = map.get_mut(&name) {
                        daemon.pid = pid;
                        daemon.last_exit_code = last_exit_code;
                    } else {
                        let for_use_by = if name.contains("com.apple") {
                            ForWhom::Apple
                        } else {
                            ForWhom::ThirdParty
                        };
                        let daemon = Daemon {
                            name: name.clone(),
                            source_path: None,
                            domain,
                            for_use_by,
                            plist: None,
                            pid,
                            last_exit_code,
                        };
                        map.insert(name, daemon);
                    }
                }
            }
        }

        Ok(Self { map })
    }

    pub fn get_match(&self, query: &str, exact: bool) -> Result<&Daemon> {
        self.select_match(self.find_matches(query, exact)?, query)
    }

    /// Fuzzy match restricted to plists loaded from angel-managed dirs.
    pub fn get_angel_match(&self, query: &str) -> Result<&Daemon> {
        let matches = self
            .find_matches(query, false)?
            .into_iter()
            .filter(|d| d.for_use_by == ForWhom::Angel)
            .collect();
        self.select_match(matches, query)
    }

    pub fn get_matches(&self, query: &str, exact: bool) -> Result<Vec<&Daemon>> {
        self.find_matches(query, exact)
    }

    fn select_match<'a>(&self, matches: Vec<&'a Daemon>, query: &str) -> Result<&'a Daemon> {
        match matches.len() {
            0 => Err(Error::NotFound(err_msg!(query))),
            1 => Ok(matches[0]),
            _ => {
                let items: Vec<String> = matches
                    .iter()
                    .map(|daemon| {
                        format!(
                            "{:<19}{}  {}",
                            styles::color_domain(&daemon.domain),
                            daemon.name,
                            styles::command(&styles::display_path(daemon, false))
                        )
                    })
                    .collect();

                let selection = dialoguer::Select::new()
                    .with_prompt(format!(
                        "Multiple daemons found matching '{}'. Select one:",
                        query
                    ))
                    .items(&items)
                    .default(0)
                    .interact()?;

                Ok(matches[selection])
            }
        }
    }

    fn find_matches(&self, query: &str, exact: bool) -> Result<Vec<&Daemon>> {
        let query_lower = if exact { String::new() } else { query.to_ascii_lowercase() };
        Ok(self
            .map
            .values()
            .filter(|daemon| {
                if exact {
                    daemon.name.eq_ignore_ascii_case(query)
                } else {
                    daemon.name.to_ascii_lowercase().contains(&query_lower)
                }
            })
            .collect())
    }
}
