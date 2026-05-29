use serde::Deserialize;

#[derive(Deserialize)]
pub struct SaveContactRequest {
    pub phone_number: String,
    pub name: String,
    pub company: Option<String>,
}
