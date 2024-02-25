use sea_orm::{EntityName, Schema};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Adobe {
    #[sea_orm(iden = "mtool_pdf_adobe")]
    Table,
    Id,
    Structure,
    #[sea_orm(iden = "asset_id")]
    AssetId,
    State,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Adobe::Table)
                    .modify_column(ColumnDef::new(Adobe::Structure).text().null())
                    .add_column(ColumnDef::new(Adobe::AssetId).text().null())
                    .add_column(ColumnDef::new(Adobe::State).integer().not_null())
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Adobe::Table).to_owned())
            .await
    }
}
