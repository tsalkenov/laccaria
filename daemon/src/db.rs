use tantivy::schema::{Schema, TEXT};
use typed_sled::{search::SearchEngine, Config};

use crate::{
    process::Process,
    state::{state_dir, DB},
};

pub type Db = typed_sled::Tree<String, Process>;
pub type Search = SearchEngine<String, Process>;

pub fn get_db() -> sled::Result<Db> {
    fn merge(
        _key: String,
        old_value: Option<Process>, // the previous value, if one existed
        merged_bytes: Process,      // the new bytes being merged in
    ) -> Option<Process> {
        // set the new value, return None to delete
        let mut ret = old_value.map_or_else(Process::default(), f)

        ret.extend_from_slice(merged_bytes);

        Some(ret)
    }
    let db = Config::new().path(state_dir().join(DB)).open()?;

    Ok(Db::open(&db, "procs"))
}

pub fn get_search_engine(db: &Db) -> anyhow::Result<Search> {
    let mut schema_builder = Schema::builder();
    let name = schema_builder.add_text_field("name", TEXT);

    Ok(SearchEngine::new_temp(
        db,
        schema_builder,
        move |_k, v| todo!(),
    )?)
}
