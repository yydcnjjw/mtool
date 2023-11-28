use mapp::provider::Res;
use mtool_core::ConfigStore;
use sea_orm::{Database, DatabaseConnection};

pub async fn create_db_conn(
    cs: Res<ConfigStore>,
) -> Result<Res<DatabaseConnection>, anyhow::Error> {
    let path = cs.get::<String>("storage.db").await?;
    let db: DatabaseConnection = Database::connect(format!("sqlite://{}?mode=rwc", path)).await?;
    Ok(Res::new(db))
}
