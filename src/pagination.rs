use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
pub struct Pagination {
    #[param(default = 1, minimum = 1)]
    pub page: Option<u64>,
    #[param(default = 10, minimum = 1, maximum = 100)]
    pub limit: Option<u64>,
}


#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_items: u64,
    pub total_pages: u64,
    pub current_page: u64,
    pub page_size: u64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total_items: u64, page: u64, page_size: u64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            (total_items as f64 / page_size as f64).ceil() as u64
        };
        
        Self {
            data,
            total_items,
            total_pages,
            current_page: page,
            page_size,
        }
    }
}
