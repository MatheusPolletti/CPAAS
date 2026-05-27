use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Whatsapp::Table)
                    .add_column(ColumnDef::new(Whatsapp::MediaUrl).string())
                    .add_column(ColumnDef::new(Whatsapp::MediaType).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Whatsapp::Table)
                    .drop_column(Whatsapp::MediaUrl)
                    .drop_column(Whatsapp::MediaType)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Whatsapp {
    Table,
    MediaUrl,
    MediaType,
}
