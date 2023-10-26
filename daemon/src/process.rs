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

impl Default for Process {
    fn default() -> Self {
        Process { pid: 0, status: Status::Dead, command: String::new(), restart: false }
    }
}

impl Process {
    pub fn find_by_name(name: String, db: &Db) -> anyhow::Result<Self>{
        db.get(&name)?.context("Process not found")
    }
}
