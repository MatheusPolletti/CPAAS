pub use sea_orm_migration::prelude::*;

mod m20260513_105338_create_users_table;
mod m20260514_123435_add_refresh_token_to_users;
mod m20260515_134044_create_sms_table;
mod m20260518_141724_create_whatsapp_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260513_105338_create_users_table::Migration),
            Box::new(m20260514_123435_add_refresh_token_to_users::Migration),
            Box::new(m20260515_134044_create_sms_table::Migration),
            Box::new(m20260518_141724_create_whatsapp_table::Migration),
        ]
    }
}
