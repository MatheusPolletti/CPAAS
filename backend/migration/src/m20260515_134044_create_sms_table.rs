use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sms::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sms::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sms::Direction).string().not_null())
                    .col(ColumnDef::new(Sms::FromNumber).string().not_null())
                    .col(ColumnDef::new(Sms::ToNumber).string().not_null())
                    .col(ColumnDef::new(Sms::Body).string())
                    .col(ColumnDef::new(Sms::Status).string())
                    .col(
                        ColumnDef::new(Sms::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Sms::UserId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sms_user_id")
                            .from(Sms::Table, Sms::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sms::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Sms {
    Table,
    Id,
    Direction,
    FromNumber,
    ToNumber,
    Body,
    Status,
    CreatedAt,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
