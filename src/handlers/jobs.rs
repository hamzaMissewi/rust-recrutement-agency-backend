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
use crate::utils::{ApiResponse, PaginationParams, PaginatedResponse};

#[derive(Debug, Deserialize)]
pub struct JobFilterQuery {
    pub client_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub location: Option<String>,
    pub job_type: Option<String>,
    pub salary_min: Option<String>,
    pub salary_max: Option<String>,
    pub search: Option<String>,
}

pub async fn get_jobs(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<JobFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let pagination = PaginationParams::new(pagination.page, pagination.limit);
    
    let mut base_query = "
        SELECT j.id, j.client_id, j.title, j.description, j.requirements, 
               j.salary_range, j.location, j.job_type, j.is_active, j.created_at, j.updated_at,
               c.company_name
        FROM job_postings j
        JOIN clients c ON j.client_id = c.id
        WHERE 1=1
    ".to_string();
    
    let mut count_query = "
        SELECT COUNT(*) as total FROM job_postings j WHERE 1=1
    ".to_string();
    
    let mut params = Vec::new();
    let mut param_count = 0;
    
    if let Some(client_id) = filters.client_id {
        param_count += 1;
        base_query.push_str(&format!(" AND j.client_id = ${}", param_count));
        count_query.push_str(&format!(" AND j.client_id = ${}", param_count));
        params.push(client_id.to_string());
    }
    
    if let Some(is_active) = filters.is_active {
        param_count += 1;
        base_query.push_str(&format!(" AND j.is_active = ${}", param_count));
        count_query.push_str(&format!(" AND j.is_active = ${}", param_count));
        params.push(is_active.to_string());
    }
    
    if let Some(location) = &filters.location {
        param_count += 1;
        let location_param = format!("%{}%", location);
        base_query.push_str(&format!(" AND j.location ILIKE ${}", param_count));
        count_query.push_str(&format!(" AND j.location ILIKE ${}", param_count));
        params.push(location_param);
    }
    
    if let Some(job_type) = &filters.job_type {
        param_count += 1;
        base_query.push_str(&format!(" AND j.job_type = ${}", param_count));
        count_query.push_str(&format!(" AND j.job_type = ${}", param_count));
        params.push(job_type.clone());
    }
    
    if let Some(search) = &filters.search {
        param_count += 1;
        let search_param = format!("%{}%", search);
        base_query.push_str(&format!(" AND (j.title ILIKE ${} OR j.description ILIKE ${})", param_count, param_count + 1));
        count_query.push_str(&format!(" AND (j.title ILIKE ${} OR j.description ILIKE ${})", param_count, param_count + 1));
        params.push(search_param.clone());
        params.push(search_param);
        param_count += 1;
    }
    
    if let Some(salary_min) = &filters.salary_min {
        param_count += 1;
        base_query.push_str(&format!(" AND j.salary_range IS NOT NULL AND CAST(SPLIT_PART(j.salary_range, '-', 1) AS INTEGER) >= ${}", param_count));
        params.push(salary_min.clone());
    }
    
    if let Some(salary_max) = &filters.salary_max {
        param_count += 1;
        base_query.push_str(&format!(" AND j.salary_range IS NOT NULL AND CAST(SPLIT_PART(j.salary_range, '-', 2) AS INTEGER) <= ${}", param_count));
        params.push(salary_max.clone());
    }
    
    base_query.push_str(&format!(" ORDER BY j.created_at DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));
    
    // Execute count query
    let total: i64 = sqlx::query_scalar(&count_query)
        .bind(&params.get(0).unwrap_or(&String::new()))
        .bind(&params.get(1).unwrap_or(&String::new()))
        .fetch_one(&pool)
        .await?;
    
    // Execute main query
    let mut query = sqlx::query(&base_query);
    for param in &params {
        query = query.bind(param);
    }
    query = query.bind(pagination.limit()).bind(pagination.offset());
    
    let rows = query.fetch_all(&pool).await?;
    
    let mut jobs = Vec::new();
    for row in rows {
        let job = JobPosting {
            id: row.get("id"),
            client_id: row.get("client_id"),
            title: row.get("title"),
            description: row.get("description"),
            requirements: row.get("requirements"),
            salary_range: row.get("salary_range"),
            location: row.get("location"),
            job_type: row.get("job_type"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        jobs.push(job);
    }
    
    let response = PaginatedResponse::new(jobs, pagination.page.unwrap_or(1), pagination.limit.unwrap_or(20), total);
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_job(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let job = sqlx::query_as!(
        JobPosting,
        r#"
        SELECT id, client_id, title, description, requirements, salary_range, location, job_type, is_active, created_at, updated_at
        FROM job_postings
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?;
    
    match job {
        Some(job) => Ok(Json(ApiResponse::success(job))),
        None => Err(AppError::NotFound),
    }
}

pub async fn create_job(
    State(pool): State<PgPool>,
    Json(mut request): Json<CreateJobRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.title.trim().is_empty() {
        return Err(AppError::BadRequest("Job title is required".to_string()));
    }
    
    if request.description.trim().is_empty() {
        return Err(AppError::BadRequest("Job description is required".to_string()));
    }
    
    if request.location.trim().is_empty() {
        return Err(AppError::BadRequest("Job location is required".to_string()));
    }
    
    // Check if client exists
    let client_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM clients WHERE id = $1",
        request.client_id
    )
    .fetch_one(&pool)
    .await?;
    
    if client_exists.unwrap_or(0) == 0 {
        return Err(AppError::BadRequest("Client not found".to_string()));
    }
    
    // Clean and deduplicate requirements
    request.requirements.sort();
    request.requirements.dedup();
    
    let job = sqlx::query_as!(
        JobPosting,
        r#"
        INSERT INTO job_postings (id, client_id, title, description, requirements, salary_range, location, job_type, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, client_id, title, description, requirements, salary_range, location, job_type, is_active, created_at, updated_at
        "#,
        Uuid::new_v4(),
        request.client_id,
        request.title.trim(),
        request.description.trim(),
        &request.requirements,
        request.salary_range,
        request.location.trim(),
        request.job_type.unwrap_or_else(|| "full-time".to_string()),
        request.is_active.unwrap_or(true)
    )
    .fetch_one(&pool)
    .await?;
    
    Ok((StatusCode::CREATED, Json(ApiResponse::success(job))))
}

pub async fn update_job(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(mut request): Json<CreateJobRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.title.trim().is_empty() {
        return Err(AppError::BadRequest("Job title is required".to_string()));
    }
    
    if request.description.trim().is_empty() {
        return Err(AppError::BadRequest("Job description is required".to_string()));
    }
    
    if request.location.trim().is_empty() {
        return Err(AppError::BadRequest("Job location is required".to_string()));
    }
    
    // Check if job exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM job_postings WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    // Check if client exists
    let client_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM clients WHERE id = $1",
        request.client_id
    )
    .fetch_one(&pool)
    .await?;
    
    if client_exists.unwrap_or(0) == 0 {
        return Err(AppError::BadRequest("Client not found".to_string()));
    }
    
    // Clean and deduplicate requirements
    request.requirements.sort();
    request.requirements.dedup();
    
    let job = sqlx::query_as!(
        JobPosting,
        r#"
        UPDATE job_postings 
        SET client_id = $1, title = $2, description = $3, requirements = $4, salary_range = $5, 
            location = $6, job_type = $7, is_active = $8, updated_at = NOW()
        WHERE id = $9
        RETURNING id, client_id, title, description, requirements, salary_range, location, job_type, is_active, created_at, updated_at
        "#,
        request.client_id,
        request.title.trim(),
        request.description.trim(),
        &request.requirements,
        request.salary_range,
        request.location.trim(),
        request.job_type.unwrap_or_else(|| "full-time".to_string()),
        request.is_active.unwrap_or(true),
        id
    )
    .fetch_one(&pool)
    .await?;
    
    Ok(Json(ApiResponse::success(job)))
}

pub async fn delete_job(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check if job exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM job_postings WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    // Check if job has active applications
    let active_applications = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM applications WHERE job_id = $1 AND status IN ('pending', 'reviewing')",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if active_applications.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Cannot delete job with active applications".to_string()));
    }
    
    let result = sqlx::query!(
        "DELETE FROM job_postings WHERE id = $1",
        id
    )
    .execute(&pool)
    .await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": true}))))
}

pub async fn get_job_applications(
    State(pool): State<PgPool>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let applications = sqlx::query_as!(
        Application,
        r#"
        SELECT a.id, a.job_id, a.worker_id, a.status, a.cover_letter, a.applied_at, a.updated_at,
               w.name as worker_name, w.email as worker_email
        FROM applications a
        JOIN workers w ON a.worker_id = w.id
        WHERE a.job_id = $1
        ORDER BY a.applied_at DESC
        "#,
        job_id
    )
    .fetch_all(&pool)
    .await?;
    
    Ok(Json(ApiResponse::success(applications)))
}
