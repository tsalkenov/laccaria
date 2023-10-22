use anyhow::Context;
use sea_orm::entity::prelude::*;

#[derive(DeriveActiveEnum, EnumIter, PartialEq, Clone, Debug)]
#[sea_orm(rs_type = "u32", db_type = "Integer")]
pub enum Status {
    Active = 1,
    Dead = 0,
}

#[derive(DeriveEntityModel, Clone, Debug)]
#[sea_orm(table_name = "process")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub pid: u32,
    #[sea_orm(unique)]
    pub name: String,
    pub status: Status,
    pub command: String,
    #[sea_orm(default_value = false)]
    pub restart: bool,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn find_by_name(name: &str, db: &DatabaseConnection) -> anyhow::Result<Self> {
        Entity::find()
            .filter(Column::Name.eq(name))
            .one(db)
            .await?
            .context("Process not found")
    }

    pub async fn find_by_pid(pid: u32, db: &DatabaseConnection) -> anyhow::Result<Self> {
        Entity::find()
            .filter(Column::Pid.eq(pid))
            .one(db)
            .await?
            .context("Process not found")
    }
}
