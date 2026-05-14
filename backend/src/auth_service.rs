use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::entities::users;
use crate::entities::users::{Entity as User, Model as UserModel};

pub struct AuthService {
    pepper: String,
    db: DatabaseConnection,
}

impl AuthService {
    pub fn new(pepper: String, db: DatabaseConnection) -> Self {
        Self { pepper, db }
    }

    fn get_hasher(&self) -> Argon2<'_> {
        Argon2::new_with_secret(
            self.pepper.as_bytes(),
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        )
        .expect("Erro ao configurar Argon2")
    }

    pub fn verify_password(&self, password: &str, hash_str: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(hash_str) else {
            return false;
        };

        let argon2 = self.get_hasher();
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<UserModel, String> {
        let user_option = User::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro de banco: {:?}", e))?;

        println!("{:?}", user_option);

        if let Some(user) = user_option {
            if self.verify_password(password, &user.password) {
                Ok(user)
            } else {
                Err("Senha incorreta".to_string())
            }
        } else {
            Err("Usuário não encontrado".to_string())
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt: SaltString = SaltString::generate(&mut OsRng);
        let argon2 = self.get_hasher();

        let password_hash: PasswordHash<'_> = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    pub async fn register_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<users::Model, sea_orm::DbErr> {
        let hash = self.hash_password(password).expect("Erro ao gerar hash");

        let new_user = users::ActiveModel {
            username: Set(username.to_owned()),
            email: Set(email.to_owned()),
            password: Set(hash),
            ..Default::default()
        };

        new_user.insert(&self.db).await
    }
}
