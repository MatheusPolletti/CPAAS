use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, IntoActiveModel};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    auth::jwt_utils::Claims,
    entities::users::{self, Entity as User, Model as UserModel},
};

#[derive(Clone)]
pub struct AuthService {
    pepper: String,
    db: DatabaseConnection,
}

impl AuthService {
    pub fn new(pepper: String, db: DatabaseConnection) -> Self {
        Self { pepper, db }
    }

    pub fn get_pepper(&self) -> &str {
        &self.pepper
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

    pub async fn refresh_session(&self, token: &str) -> Result<(String, String), String> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.pepper.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| "Token de refresh inválido ou expirado".to_string())?;

        if token_data.claims.token_type != "refresh" {
            return Err("Tipo de token inválido".to_string());
        }

        let user_id = token_data.claims.sub;

        let user_option = User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro de banco: {:?}", e))?;

        if let Some(user) = user_option {
            let db_token = user.refresh_token.as_deref().unwrap_or("");
            if db_token != token {
                return Err("Token de refresh revogado ou substituído".to_string());
            }

            let (new_access, new_refresh) = self.generate_tokens(user.id)?;

            let mut user_active: users::ActiveModel = user.into_active_model();
            user_active.refresh_token = Set(Some(new_refresh.clone()));

            user_active
                .update(&self.db)
                .await
                .map_err(|e| format!("Erro ao salvar novo token: {:?}", e))?;

            Ok((new_access, new_refresh))
        } else {
            Err("Usuário não encontrado".to_string())
        }
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

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(UserModel, String, String), String> {
        let user_option = User::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro de banco: {:?}", e))?;

        if let Some(user) = user_option {
            if self.verify_password(password, &user.password) {
                let (access, refresh) = self.generate_tokens(user.id)?;

                let mut user_active: users::ActiveModel = user.into_active_model();

                user_active.refresh_token = Set(Some(refresh.clone()));

                let updated_user = user_active
                    .update(&self.db)
                    .await
                    .map_err(|e| format!("Erro ao salvar token: {:?}", e))?;

                Ok((updated_user, access, refresh))
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

    pub fn generate_tokens(&self, user_id: i32) -> Result<(String, String), String> {
        let now = Utc::now();

        let access_exp = (now + Duration::minutes(15)).timestamp() as usize;
        let access_claims = Claims {
            sub: user_id,
            exp: access_exp,
            iat: now.timestamp() as usize,
            token_type: "access".to_string(),
        };

        let refresh_exp = (now + Duration::days(7)).timestamp() as usize;
        let refresh_claims = Claims {
            sub: user_id,
            exp: refresh_exp,
            iat: now.timestamp() as usize,
            token_type: "refresh".to_string(),
        };

        let encoding_key = EncodingKey::from_secret(self.pepper.as_bytes());

        let access_token = encode(&Header::default(), &access_claims, &encoding_key)
            .map_err(|e| format!("Falha ao gerar access token: {}", e))?;

        let refresh_token = encode(&Header::default(), &refresh_claims, &encoding_key)
            .map_err(|e| format!("Falha ao gerar refresh token: {}", e))?;

        Ok((access_token, refresh_token))
    }

    pub async fn logout(&self, user_id: i32) -> Result<(), String> {
        let user_option = User::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro de banco: {:?}", e))?;

        if let Some(user) = user_option {
            let mut user_active: users::ActiveModel = user.into_active_model();
            user_active.refresh_token = Set(None);

            user_active
                .update(&self.db)
                .await
                .map_err(|e| format!("Erro ao fazer logout: {:?}", e))?;

            Ok(())
        } else {
            Err("Usuário não encontrado".to_string())
        }
    }
}
