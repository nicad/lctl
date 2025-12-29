use std::path::PathBuf;

pub fn get_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
