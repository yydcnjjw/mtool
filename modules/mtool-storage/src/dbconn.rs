use mapp::{anyhow, prelude::*};
use mtool_core::ConfigStore;
use sea_orm::{Database, DatabaseConnection};

#[allow(unused)]
pub async fn create_dbconn(
    cs: Res<ConfigStore>,
) -> Result<Res<DatabaseConnection>, anyhow::Error> {
    Ok(Res::new(create_dbconn_inner(cs).await?))
}

pub(crate) async fn create_dbconn_inner(
    cs: Res<ConfigStore>,
) -> Result<DatabaseConnection, anyhow::Error> {
    let path = cs.get::<String>("storage.db")?;
    let db: DatabaseConnection = Database::connect(format!("sqlite://{}?mode=rwc", path)).await?;
    Ok(db)
}
