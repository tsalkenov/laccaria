use std::{path::PathBuf, env, fs};

const STATE_DIR: &str = ".laccaria";
pub const ACCESS_LOG: &str = "access.log";
pub const ERROR_LOG: &str = "error.log";

pub fn state_dir() -> PathBuf {
    let Ok(home) = env::var("HOME") else {
        log::error!("HOME variable not specified");
        std::process::exit(1)
    };
    let state_dir = PathBuf::from(home).join(STATE_DIR);

    if !state_dir.exists() {
        if let Err(e) = fs::create_dir_all(&state_dir) {
            log::error!("Failed to create state folder: {e}");
            std::process::exit(1)
        }
    }
    state_dir

}
