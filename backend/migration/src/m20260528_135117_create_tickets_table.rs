use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tickets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tickets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tickets::Status).string().not_null())
                    .col(ColumnDef::new(Tickets::Subject).string())
                    .col(
                        ColumnDef::new(Tickets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Tickets::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Tickets::ContactId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tickets_contact_id")
                            .from(Tickets::Table, Tickets::ContactId)
                            .to(Contacts::Table, Contacts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Tickets::UserId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tickets_user_id")
                            .from(Tickets::Table, Tickets::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tickets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tickets {
    Table,
    Id,
    Status,
    Subject,
    CreatedAt,
    UpdatedAt,
    ContactId,
    UserId,
}

#[derive(DeriveIden)]
enum Contacts {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
