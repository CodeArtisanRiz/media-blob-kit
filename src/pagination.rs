use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct Pagination {
    #[param(default = 1, minimum = 1)]
    pub page: Option<u64>,
    #[param(default = 10, minimum = 1, maximum = 100)]
    pub limit: Option<u64>,
}

impl Pagination {
    pub fn offset(&self) -> u64 {
        (self.page.unwrap_or(1) - 1) * self.limit.unwrap_or(10)
    }

    pub fn limit(&self) -> u64 {
        self.limit.unwrap_or(10)
    }
}
