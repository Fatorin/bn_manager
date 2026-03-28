use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::database::mysql_pool;
use crate::model::pagination::{paginate, PaginationResult};
use crate::model::score::Score;

#[derive(Deserialize)]
pub struct ScoreQuery {
    pub category: Option<String>,
    pub server: Option<String>,
    pub name: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_scores(Query(params): Query<ScoreQuery>) -> impl IntoResponse {
    let mut limit = params.limit.unwrap_or(25);
    if limit <= 0 {
        limit = 25;
    }
    if limit > 50 {
        limit = 50;
    }

    let mut offset = params.offset.unwrap_or(0);
    if offset < 0 {
        offset = 0;
    }

    let sort_by = match params.sort_by.as_deref() {
        Some("name") => "name",
        _ => "score",
    };

    let sort_order = match params.sort_order.as_deref().map(|s| s.to_uppercase()).as_deref() {
        Some("ASC") => "ASC",
        _ => "DESC",
    };

    let mut conditions = vec!["1=1".to_string()];
    let mut count_args: Vec<String> = Vec::new();

    if let Some(ref category) = params.category {
        if !category.is_empty() {
            conditions.push("category = ?".to_string());
            count_args.push(category.clone());
        }
    }
    if let Some(ref server) = params.server {
        if !server.is_empty() {
            conditions.push("server = ?".to_string());
            count_args.push(server.clone());
        }
    }
    if let Some(ref name) = params.name {
        if !name.is_empty() {
            conditions.push("name LIKE ?".to_string());
            count_args.push(format!("%{}%", name));
        }
    }

    let where_clause = conditions.join(" AND ");
    let pool = mysql_pool();

    // Count query
    let count_query = format!("SELECT COUNT(*) FROM scores WHERE {}", where_clause);
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
    for arg in &count_args {
        count_q = count_q.bind(arg);
    }

    let total = match count_q.fetch_one(pool).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Data query
    let data_query = format!(
        "SELECT id, name, score FROM scores WHERE {} ORDER BY {} {} LIMIT ? OFFSET ?",
        where_clause, sort_by, sort_order
    );
    let mut data_q = sqlx::query_as::<_, Score>(&data_query);
    for arg in &count_args {
        data_q = data_q.bind(arg);
    }
    data_q = data_q.bind(limit).bind(offset);

    let scores = match data_q.fetch_all(pool).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let (pages, current_page, has_next) = paginate(total, limit, offset);

    Json(PaginationResult {
        total,
        limit,
        offset,
        page: current_page,
        pages,
        has_next,
        data: scores,
    })
    .into_response()
}
