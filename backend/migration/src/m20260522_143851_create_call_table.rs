use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Calls::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Calls::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Calls::CallSid)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Calls::FromNumber).string().not_null())
                    .col(ColumnDef::new(Calls::ToNumber).string().not_null())
                    .col(ColumnDef::new(Calls::Direction).string().not_null())
                    .col(ColumnDef::new(Calls::Status).string().not_null())
                    .col(ColumnDef::new(Calls::Duration).integer())
                    .col(
                        ColumnDef::new(Calls::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Calls::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Calls::UserId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_calls_user_id")
                            .from(Calls::Table, Calls::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Calls::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Calls {
    Table,
    Id,
    CallSid,
    FromNumber,
    ToNumber,
    Direction,
    Status,
    Duration,
    CreatedAt,
    UpdatedAt,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
