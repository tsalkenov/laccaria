use std::sync::Arc;

use sled::Config;

use crate::state::{state_dir, DB};

pub type Db = Arc<sled::Db>;

pub fn get_db() -> anyhow::Result<Db> {
    #[cfg(feature = "clean")]
    for file in std::fs::read_dir(state_dir().join(DB))? {
        let path = file?.path();

        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }

    let db = Config::new().path(state_dir().join(DB)).open()?;

    Ok(Arc::new(db))
}
