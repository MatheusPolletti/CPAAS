use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contacts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Contacts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Contacts::PhoneNumber)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Contacts::Name).string().not_null())
                    .col(ColumnDef::new(Contacts::Company).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contacts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Contacts {
    Table,
    Id,
    PhoneNumber,
    Name,
    Company,
}
