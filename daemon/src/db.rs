use typed_sled::Config;

use crate::{
    process::Process,
    state::{state_dir, DB},
};

pub type Db = typed_sled::Tree<String, Process>;

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

    let db = Config::new().path(dbg!(state_dir().join(DB))).open()?;
    let tree = Db::open(&db, "procs");

    Ok(tree)
}
