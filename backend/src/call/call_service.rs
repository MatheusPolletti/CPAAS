use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, QueryFilter};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entities::{calls, calls::Entity as CallEntity};
use sea_orm::{EntityTrait, QueryOrder};

#[derive(Clone)]
pub struct CallService {
    twilio_account_sid: String,
    twilio_phone_number: String,
    twilio_api_key_sid: String,
    twilio_api_key_secret: String,
    twiml_app_sid: String,
    db: DatabaseConnection,
}

impl CallService {
    pub fn new(
        sid: String,
        phone: String,
        api_key_sid: String,
        api_key_secret: String,
        twiml_app_sid: String,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            twilio_account_sid: sid,
            twilio_phone_number: phone,
            twilio_api_key_sid: api_key_sid,
            twilio_api_key_secret: api_key_secret,
            twiml_app_sid,
            db,
        }
    }

    pub fn get_caller_id(&self) -> &str {
        &self.twilio_phone_number
    }

    pub fn generate_voice_token(&self, identity: &str) -> Result<String, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Erro ao gerar timestamp".to_string())?
            .as_secs() as usize;

        let exp = now + 3600;

        let jti = format!("{}-{}", self.twilio_api_key_sid, now);

        let grants = TokenGrants {
            identity: identity.to_string(),
            voice: VoiceGrant {
                outgoing: Some(OutgoingGrant {
                    application_sid: self.twiml_app_sid.clone(),
                }),
                incoming: None,
            },
        };

        let claims = TwilioClaims {
            jti,
            iss: self.twilio_api_key_sid.clone(),
            sub: self.twilio_account_sid.clone(),
            exp,
            grants,
        };

        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".to_string());
        header.cty = Some("twilio-fpa;v=1".to_string());

        let clean_secret = self.twilio_api_key_secret.trim();

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(clean_secret.as_bytes()),
        )
        .map_err(|e| format!("Erro ao gerar token: {}", e))
    }

    pub async fn update_call_status(
        &self,
        sid_to_search: &str,
        status: &str,
    ) -> Result<(), String> {
        use sea_orm::ColumnTrait;

        let call_opt = CallEntity::find()
            .filter(calls::Column::CallSid.eq(sid_to_search))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro ao buscar ligação no banco: {}", e))?;

        if let Some(call) = call_opt {
            let mut active_call: calls::ActiveModel = call.into();
            active_call.status = Set(status.to_string());

            active_call
                .update(&self.db)
                .await
                .map_err(|e| format!("Erro ao atualizar status: {}", e))?;
        }

        Ok(())
    }

    pub async fn register_outbound_webrtc(
        &self,
        sid: &str,
        to: &str,
        client_from: Option<String>,
    ) -> Result<(), String> {
        let mut user_id = None;

        if let Some(from_str) = client_from {
            if from_str.starts_with("client:user-") {
                let id_str = from_str.replace("client:user-", "");
                if let Ok(id) = id_str.parse::<i32>() {
                    user_id = Some(id);
                }
            }
        }

        let new_call = calls::ActiveModel {
            call_sid: Set(sid.to_string()),
            from_number: Set(self.twilio_phone_number.clone()),
            to_number: Set(to.to_string()),
            direction: Set("outbound".to_string()),
            status: Set("in-progress".to_string()),
            user_id: Set(user_id),
            ..Default::default()
        };

        new_call
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar outbound no banco: {}", e))?;

        Ok(())
    }

    pub async fn register_inbound(&self, sid: &str, from: &str, to: &str) -> Result<(), String> {
        let new_call = calls::ActiveModel {
            call_sid: Set(sid.to_string()),
            from_number: Set(from.to_string()),
            to_number: Set(to.to_string()),
            direction: Set("inbound".to_string()),
            status: Set("in-progress".to_string()),
            ..Default::default()
        };

        new_call
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar inbound no banco: {}", e))?;

        Ok(())
    }

    pub async fn get_call_history(&self) -> Result<Vec<calls::Model>, String> {
        CallEntity::find()
            .order_by_desc(calls::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro ao buscar o histórico de ligações: {}", e))
    }
}

#[derive(Serialize)]
struct VoiceGrant {
    #[serde(skip_serializing_if = "Option::is_none")]
    outgoing: Option<OutgoingGrant>,

    #[serde(skip_serializing_if = "Option::is_none")]
    incoming: Option<IncomingGrant>,
}

#[derive(Serialize)]
struct OutgoingGrant {
    application_sid: String,
}

#[derive(Serialize)]
struct IncomingGrant {
    allow: bool,
}

#[derive(Serialize)]
struct TokenGrants {
    identity: String,
    voice: VoiceGrant,
}

#[derive(Serialize)]
struct TwilioClaims {
    jti: String,
    iss: String,
    sub: String,
    exp: usize,
    grants: TokenGrants,
}
