use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Whatsapp::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Whatsapp::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Whatsapp::Direction).string().not_null())
                    .col(ColumnDef::new(Whatsapp::FromNumber).string().not_null())
                    .col(ColumnDef::new(Whatsapp::ToNumber).string().not_null())
                    .col(ColumnDef::new(Whatsapp::Body).string())
                    .col(ColumnDef::new(Whatsapp::Status).string().not_null())
                    .col(ColumnDef::new(Whatsapp::SenderName).string())
                    .col(ColumnDef::new(Whatsapp::TwilioSid).string())
                    .col(
                        ColumnDef::new(Whatsapp::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Whatsapp::UserId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_whatsapp_user_id")
                            .from(Whatsapp::Table, Whatsapp::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Whatsapp::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Whatsapp {
    Table,
    Id,
    Direction,
    FromNumber,
    ToNumber,
    Body,
    Status,
    SenderName,
    TwilioSid,
    CreatedAt,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
