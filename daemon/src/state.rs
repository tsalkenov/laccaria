use std::{env, fs, io, path::PathBuf};

const STATE_DIR: &str = ".laccaria";
pub const LOG: &str = "access.log";
pub const DB: &str = "db";

pub fn state_dir() -> PathBuf {
    let Ok(home) = env::var("HOME") else {
        log::error!("HOME variable not specified");
        std::process::exit(1)
    };
    PathBuf::from(home).join(STATE_DIR)
}

pub fn init_state() -> Result<(), io::Error> {
    let state_dir = state_dir();
    fs::create_dir_all(&state_dir)?;
    fs::create_dir_all(state_dir.join(DB)).expect("fuck");

    let derivatives = vec![state_dir.join(LOG)];
    for derivative in derivatives {
        if !derivative.exists() {
            if let Err(e) = fs::File::create(&derivative) {
                log::error!("Failed to create state file: {e}");
                std::process::exit(1)
            }
        }
    }

    Ok(())
}
