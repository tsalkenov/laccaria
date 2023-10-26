use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    Active = 1,
    Dead = 0,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    pub name: String,
    pub status: Status,
    pub command: String,
    pub restart: bool,
}
