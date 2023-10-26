use tantivy::schema::{Schema, TEXT};
use typed_sled::{search::SearchEngine, Config};

use crate::{
    process::Process,
    state::{state_dir, DB},
};

pub type Db = typed_sled::Tree<u32, Process>;
pub type Search = SearchEngine<u32, Process>;

pub fn get_db() -> sled::Result<Db> {
    let db = Config::new().path(state_dir().join(DB)).open()?;

    Ok(Db::open(&db, "procs"))
}

pub fn get_search_engine(db: &Db) -> anyhow::Result<Search> {
    let mut schema_builder = Schema::builder();
    let name = schema_builder.add_text_field("name", TEXT);

    Ok(SearchEngine::new_temp(db, schema_builder, move |_k, v| {
        tantivy::doc!(
            name => v.name.to_owned()
        )
    })?)
}
