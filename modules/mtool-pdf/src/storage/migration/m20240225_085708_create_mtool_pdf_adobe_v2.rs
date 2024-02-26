use sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Adobe {
    #[sea_orm(iden = "mtool_pdf_adobe")]
    Table,
    #[sea_orm(iden = "asset_id")]
    AssetId,
    #[sea_orm(iden = "media_type")]
    MediaType,
    #[sea_orm(iden = "upload_uri")]
    UploadUri,
    State,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        const TABLE: &'static str = "mtool_pdf_adobe";

        if matches!(manager.get_database_backend(), DatabaseBackend::Sqlite) {
            if !manager.has_column(TABLE, "asset_id").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .add_column(ColumnDef::new(Adobe::AssetId).text().null())
                            .take(),
                    )
                    .await?;
            }

            if !manager.has_column(TABLE, "media_type").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .add_column(ColumnDef::new(Adobe::MediaType).text().null())
                            .take(),
                    )
                    .await?;
            }

            if !manager.has_column(TABLE, "upload_uri").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .add_column(ColumnDef::new(Adobe::UploadUri).text().null())
                            .take(),
                    )
                    .await?;
            }

            if !manager.has_column(TABLE, "state").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .add_column(
                                ColumnDef::new(Adobe::State).integer().not_null().default(0),
                            )
                            .take(),
                    )
                    .await?;
            }

            manager.get_connection().execute_unprepared(r#"
PRAGMA writable_schema = 1;
UPDATE SQLITE_MASTER SET SQL = 'CREATE TABLE IF NOT EXISTS "mtool_pdf_adobe" ( "id" blob NOT NULL PRIMARY KEY, "structure" text NULL , "asset_id" text NULL, "media_type" text NULL, "upload_uri" text NULL, "state" integer NOT NULL DEFAULT 0)' WHERE NAME = 'mtool_pdf_adobe';
PRAGMA writable_schema = 0;
"#).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        const TABLE: &'static str = "mtool_pdf_adobe";

        if matches!(manager.get_database_backend(), DatabaseBackend::Sqlite) {
            if manager.has_column(TABLE, "asset_id").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .drop_column(Adobe::AssetId)
                            .take(),
                    )
                    .await?;
            }

            if manager.has_column(TABLE, "media_type").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .drop_column(Adobe::MediaType)
                            .take(),
                    )
                    .await?;
            }

            if manager.has_column(TABLE, "upload_uri").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .drop_column(Adobe::UploadUri)
                            .take(),
                    )
                    .await?;
            }

            if manager.has_column(TABLE, "state").await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Adobe::Table)
                            .drop_column(Adobe::State)
                            .take(),
                    )
                    .await?;
            }

            manager.get_connection().execute_unprepared(r#"
PRAGMA writable_schema = 1;
UPDATE SQLITE_MASTER SET SQL = 'CREATE TABLE IF NOT EXISTS "mtool_pdf_adobe" ( "id" blob NOT NULL PRIMARY KEY, "structure" text NOT NULL )' WHERE NAME = 'mtool_pdf_adobe';
PRAGMA writable_schema = 0;
"#).await?;
        }

        Ok(())
    }
}
