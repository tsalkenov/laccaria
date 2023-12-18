use anyhow::Context;
use bitcode::{decode, Decode, Encode};

use crate::db::Db;

#[derive(Encode, Decode, Debug)]
pub enum Status {
    Active = 1,
    Dead = 0,
}

#[derive(Encode, Decode, Debug)]
pub struct Process {
    pub pid: u32,
    pub status: Status,
    pub command: String,
    pub restart: bool,
}

impl Process {
    pub fn get<S: AsRef<[u8]>>(name: S, db: &Db) -> anyhow::Result<Self> {
        decode(&db.get(name)?.context("Process not found")?).map_err(Into::into)
    }
}
