use std::time::Duration;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};

use crate::{
    process,
    state::{state_dir, DB},
};

pub async fn get_db() -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(dbg!(format!(
        "sqlite:///{}",
        state_dir().join(DB).to_string_lossy()
    )));
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    Database::connect(opt).await
}

pub async fn setup_db(db: &DatabaseConnection) {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    #[cfg(feature = "clean")]
    db.execute(builder.build(sea_orm::sea_query::TableDropStatement::new().table(process::Entity)))
        .await
        .expect("how the fuck did you fail to drop table");
    db.execute(
        builder.build(
            schema
                .create_table_from_entity(process::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .expect("Failed to setup database schema");
}

#[async_std::test]
async fn test_db() {
    use crate::state::init_state;

    init_state().expect("Failed to initialize state");

    let db = get_db().await.expect("Failed to connect to db");
    assert!(db.ping().await.is_ok());

    db.clone()
        .close()
        .await
        .expect("Failed to clsoe connection to db");
    assert!(matches!(db.ping().await, Err(DbErr::ConnectionAcquire(_))));
}
