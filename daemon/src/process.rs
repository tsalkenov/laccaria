use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::db::Db;

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    Active = 1,
    Dead = 0,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    pub pid: u32,
    pub status: Status,
    pub command: String,
    pub restart: bool,
}

impl Process {
    pub fn get<'a, S: Into<&'a String>>(name: S, db: &Db) -> anyhow::Result<Self> {
        db.get(name.into())?.context("Process not found")
    }
}
