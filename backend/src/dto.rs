#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
}
