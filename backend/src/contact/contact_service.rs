use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::entities::contacts;

#[derive(Clone)]
pub struct ContactService {
    db: DatabaseConnection,
}

impl ContactService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn save_contact_name(&self, phone: &str, name: &str) -> Result<(), String> {
        let existing = contacts::Entity::find()
            .filter(contacts::Column::PhoneNumber.eq(phone))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        if let Some(contact) = existing {
            let mut active_contact: contacts::ActiveModel = contact.into();
            active_contact.name = Set(name.to_string());
            active_contact
                .update(&self.db)
                .await
                .map_err(|e| format!("Erro ao atualizar: {:?}", e))?;
        } else {
            let new_contact = contacts::ActiveModel {
                phone_number: Set(phone.to_string()),
                name: Set(name.to_string()),
                ..Default::default()
            };
            new_contact
                .insert(&self.db)
                .await
                .map_err(|e| format!("Erro ao inserir: {:?}", e))?;
        }

        Ok(())
    }
}
