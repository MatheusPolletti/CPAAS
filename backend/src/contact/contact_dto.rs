use serde::Deserialize;

#[derive(Deserialize)]
pub struct SaveContactRequest {
    pub phone_number: String,
    pub name: String,
}
