use std::sync::Arc;

use typed_sled::Config;

use crate::{
    process::Process,
    state::{state_dir, DB},
};

type Inner = typed_sled::Tree<String, Process>;
pub type Db = Arc<Inner>;

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
    let tree = Arc::new(Inner::open(&db, "procs"));

    Ok(tree)
}
