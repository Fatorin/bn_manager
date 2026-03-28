use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PaginationResult<T: Serialize> {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub page: i64,
    pub pages: i64,
    pub has_next: bool,
    pub data: Vec<T>,
}

pub fn paginate(total: i64, limit: i64, offset: i64) -> (i64, i64, bool) {
    let pages = if limit > 0 {
        (total as f64 / limit as f64).ceil() as i64
    } else {
        0
    };
    let current_page = (offset / limit) + 1;
    let has_next = offset + limit < total;
    (pages, current_page, has_next)
}
