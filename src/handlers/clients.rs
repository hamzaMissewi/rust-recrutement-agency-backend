use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::error::AppError;
use crate::utils::{ApiResponse, PaginationParams, PaginatedResponse, validate_email, validate_phone};

#[derive(Debug, Deserialize)]
pub struct ClientFilterQuery {
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn get_clients(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<ClientFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let pagination = PaginationParams::new(pagination.page, pagination.limit);
    
    let mut base_query = "
        SELECT id, company_name, email, phone, created_at, updated_at
        FROM clients
        WHERE 1=1
    ".to_string();
    
    let mut count_query = "
        SELECT COUNT(*) as total FROM clients WHERE 1=1
    ".to_string();
    
    let mut params = Vec::new();
    let mut param_count = 0;
    
    if let Some(search) = &filters.search {
        param_count += 1;
        let search_param = format!("%{}%", search);
        base_query.push_str(&format!(" AND (company_name ILIKE ${} OR email ILIKE ${})", param_count, param_count + 1));
        count_query.push_str(&format!(" AND (company_name ILIKE ${} OR email ILIKE ${})", param_count, param_count + 1));
        params.push(search_param.clone());
        params.push(search_param);
        param_count += 1;
    }
    
    base_query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));
    
    // Execute count query
    let total: i64 = sqlx::query_scalar(&count_query)
        .bind(&params[0])
        .bind(&params.get(1).unwrap_or(&String::new()))
        .fetch_one(&pool)
        .await?;
    
    // Execute main query
    let mut query = sqlx::query_as::<_, Client>(&base_query);
    for (i, param) in params.iter().enumerate() {
        query = query.bind(param);
    }
    query = query.bind(pagination.limit()).bind(pagination.offset());
    
    let clients = query.fetch_all(&pool).await?;
    
    let response = PaginatedResponse::new(clients, pagination.page.unwrap_or(1), pagination.limit.unwrap_or(20), total);
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_client(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = sqlx::query_as!(
        Client,
        r#"
        SELECT id, company_name, email, phone, created_at, updated_at
        FROM clients
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?;
    
    match client {
        Some(client) => Ok(Json(ApiResponse::success(client))),
        None => Err(AppError::NotFound),
    }
}

pub async fn create_client(
    State(pool): State<PgPool>,
    Json(mut request): Json<CreateClientRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.company_name.trim().is_empty() {
        return Err(AppError::BadRequest("Company name is required".to_string()));
    }
    
    if !validate_email(&request.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    
    if let Some(phone) = &request.phone {
        if !validate_phone(phone) {
            return Err(AppError::BadRequest("Invalid phone format".to_string()));
        }
    }
    
    // Check if email already exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM clients WHERE email = $1",
        request.email
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Email already exists".to_string()));
    }
    
    let client = sqlx::query_as!(
        Client,
        r#"
        INSERT INTO clients (id, company_name, email, phone)
        VALUES ($1, $2, $3, $4)
        RETURNING id, company_name, email, phone, created_at, updated_at
        "#,
        Uuid::new_v4(),
        request.company_name.trim(),
        request.email.trim().to_lowercase(),
        request.phone
    )
    .fetch_one(&pool)
    .await?;
    
    Ok((StatusCode::CREATED, Json(ApiResponse::success(client))))
}

pub async fn update_client(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(mut request): Json<CreateClientRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.company_name.trim().is_empty() {
        return Err(AppError::BadRequest("Company name is required".to_string()));
    }
    
    if !validate_email(&request.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    
    if let Some(phone) = &request.phone {
        if !validate_phone(phone) {
            return Err(AppError::BadRequest("Invalid phone format".to_string()));
        }
    }
    
    // Check if client exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM clients WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    // Check if email already exists for another client
    let email_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM clients WHERE email = $1 AND id != $2",
        request.email,
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if email_exists.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Email already exists".to_string()));
    }
    
    let client = sqlx::query_as!(
        Client,
        r#"
        UPDATE clients 
        SET company_name = $1, email = $2, phone = $3, updated_at = NOW()
        WHERE id = $4
        RETURNING id, company_name, email, phone, created_at, updated_at
        "#,
        request.company_name.trim(),
        request.email.trim().to_lowercase(),
        request.phone,
        id
    )
    .fetch_one(&pool)
    .await?;
    
    Ok(Json(ApiResponse::success(client)))
}

pub async fn delete_client(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check if client exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM clients WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    // Check if client has active jobs
    let active_jobs = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM job_postings WHERE client_id = $1 AND is_active = true",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if active_jobs.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Cannot delete client with active job postings".to_string()));
    }
    
    let result = sqlx::query!(
        "DELETE FROM clients WHERE id = $1",
        id
    )
    .execute(&pool)
    .await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": true}))))
}
