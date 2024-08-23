use mapp::provider::Res;
use mtool_core::ConfigStore;
use sea_orm::{Database, DatabaseConnection};

pub async fn create_db_conn(
    cs: Res<ConfigStore>,
) -> Result<Res<DatabaseConnection>, anyhow::Error> {
    Ok(Res::new(create_db_conn_inner(cs).await?))
}

pub(crate) async fn create_db_conn_inner(
    cs: Res<ConfigStore>,
) -> Result<DatabaseConnection, anyhow::Error> {
    let path = cs.get::<String>("storage.db").await?;
    let db: DatabaseConnection = Database::connect(format!("sqlite://{}?mode=rwc", path)).await?;
    Ok(db)
}
