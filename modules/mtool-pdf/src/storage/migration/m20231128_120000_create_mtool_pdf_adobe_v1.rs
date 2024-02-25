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
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Adobe::Table)
                    .col(ColumnDef::new(Adobe::Id).binary().primary_key())
                    .col(ColumnDef::new(Adobe::Structure).text().not_null())
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
