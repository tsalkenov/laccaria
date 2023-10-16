use sea_orm::entity::prelude::*;

#[derive(DeriveEntityModel, Clone, Debug)]
#[sea_orm(table_name = "process")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub pid: u32,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
