use crate::config::Config;
use crate::error::{Error, Result, err_msg};
use crate::output::stderr;
use crate::parser::Parser;
use crate::styles::styles;
use crate::types::{Daemon, Domain, ForWhom, Plist};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

struct PlistDir {
    path: PathBuf,
    domain: Domain,
    for_use_by: ForWhom,
    is_system_monitored: bool,
}

fn get_plist_dirs(config: &Config, user_uid: u32) -> Vec<PlistDir> {
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
    dirs.extend(vec![
        PlistDir {
            path: home.join("Library/LaunchAgents"),
            domain: Domain::User(user_uid),
            for_use_by: ForWhom::User,
            is_system_monitored: true,
        },
        PlistDir {
            path: home.join(".config/angel/user"),
            domain: Domain::User(user_uid),
            for_use_by: ForWhom::Angel,
            is_system_monitored: false,
        },
        PlistDir {
            path: home.join(".config/angel/system"),
            domain: Domain::System,
            for_use_by: ForWhom::Angel,
            is_system_monitored: false,
        },
        PlistDir {
            path: home.join(".config/angel/gui"),
            domain: Domain::Gui(user_uid),
            for_use_by: ForWhom::Angel,
            is_system_monitored: false,
        },
    ]);

    // Add user-defined directories from config
    for cfg_dir in config.get_directories() {
        // Convert config domain (may have placeholder uid) to Domain with correct uid
        let domain = match &cfg_dir.domain {
            Domain::System => Domain::System,
            Domain::User(_) => Domain::User(user_uid),
            Domain::Gui(_) => Domain::Gui(user_uid),
            Domain::Unknown => Domain::Unknown,
        };
        dirs.push(PlistDir {
            path: PathBuf::from(&cfg_dir.path),
            domain,
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
    pub fn new(config: &Config, uid: u32) -> Result<Self> {
        let plist_dirs = get_plist_dirs(config, uid);
        let mut map: HashMap<String, Daemon> = HashMap::new();

        // Scan plist directories
        for plist_dir in &plist_dirs {
            let pattern = format!("{}/*.plist", plist_dir.path.display());
            let matches = glob::glob(&pattern).unwrap_or_else(|_| glob::glob("").unwrap());

            for entry in matches.flatten() {
                let plist_uid = match plist_dir.domain {
                    Domain::User(uid) | Domain::Gui(uid) => uid,
                    _ => 0,
                };
                if let Ok(content) = std::fs::read(&entry) {
                    if let Ok(plist_data) = plist::from_bytes::<Plist>(&content) {
                        // pick source path for duplicates
                        let found_daemon = map.get(&plist_data.label.clone().unwrap_or_default());
                        let found_path = found_daemon.and_then(|daemon| daemon.source_path.clone());
                        let source_path = match found_path {
                            Some(found_path) => match () {
                                _ if entry.is_symlink() => entry,
                                _ if found_path.is_symlink() => found_path,
                                _ if plist_dir.is_system_monitored => found_path,
                                _ => entry,
                            },
                            None => entry,
                        };

                        let daemon = Daemon::from_plist(
                            plist_data,
                            Some(source_path),
                            plist_dir.domain.clone(),
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
                thread::spawn(move || match Parser::parse_print_domain(&domain) {
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
                        let daemon = Daemon::new(
                            name.clone(),
                            None,
                            domain.clone(),
                            for_use_by,
                            None,
                            pid,
                            last_exit_code,
                        );
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

    /// Fuzzy match restricted to plists loaded from `~/.config/angel/{user,system,gui}`.
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
                            styles::command(&styles::display_path(&daemon, false))
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
        let pattern = if exact {
            format!("^{}$", regex::escape(query))
        } else {
            format!("(?i){}", regex::escape(query))
        };

        match Regex::new(&pattern) {
            Ok(regex) => {
                Ok(self.map.values().filter(|daemon| regex.is_match(&daemon.name)).collect())
            }
            Err(e) => Err(Error::InvalidInput(err_msg!("Invalid regex: {}", e))),
        }
    }
}
