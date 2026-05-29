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

    pub async fn save_contact_name(
        &self,
        phone: &str,
        name: &str,
        company: Option<&str>,
    ) -> Result<(), String> {
        let existing = contacts::Entity::find()
            .filter(contacts::Column::PhoneNumber.eq(phone))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        if existing.is_some() {
            return Err("Este número de telefone já está cadastrado.".to_string());
        }

        let new_contact = contacts::ActiveModel {
            phone_number: Set(phone.to_string()),
            name: Set(name.to_string()),
            company: Set(company.map(|value| value.to_string())),
            ..Default::default()
        };

        new_contact
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao inserir contato: {:?}", e))?;

        Ok(())
    }

    pub async fn get_contacts(&self) -> Result<Vec<contacts::Model>, String> {
        let all_contacts = contacts::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco ao listar contatos: {:?}", e))?;

        Ok(all_contacts)
    }
}
