use mapp::{define_label, prelude::*};
use mtool_core::ConfigStore;
use parking_lot::Mutex;
use sea_orm_migration::{MigrationTrait, MigratorTrait};

use crate::db_conn::create_db_conn_inner;

static MIGRATIONS: Mutex<Vec<Box<dyn MigrationTrait>>> = Mutex::new(Vec::new());

pub struct Migrator;

pub fn add_migration<T>(migration: T)
where
    T: MigrationTrait + 'static,
{
    MIGRATIONS.lock().push(Box::new(migration));
}

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        std::mem::take(&mut MIGRATIONS.lock())
    }
}

define_label!(
    pub enum DBMigrationStage {
        Register,
        Migrate,
    }
);

pub async fn migrate(cs: Res<ConfigStore>) -> Result<(), anyhow::Error> {
    let db = create_db_conn_inner(cs).await?;
    Migrator::up(&db, None).await?;
    Ok(())
}
