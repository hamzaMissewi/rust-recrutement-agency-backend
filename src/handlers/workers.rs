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
pub struct WorkerFilterQuery {
    pub search: Option<String>,
    pub skills: Option<Vec<String>>,
    pub min_experience: Option<i32>,
    pub max_experience: Option<i32>,
}

pub async fn get_workers(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<WorkerFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let pagination = PaginationParams::new(pagination.page, pagination.limit);
    
    let mut base_query = "
        SELECT id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        FROM workers
        WHERE 1=1
    ".to_string();
    
    let mut count_query = "
        SELECT COUNT(*) as total FROM workers WHERE 1=1
    ".to_string();
    
    let mut params = Vec::new();
    let mut param_count = 0;
    
    if let Some(search) = &filters.search {
        param_count += 1;
        let search_param = format!("%{}%", search);
        base_query.push_str(&format!(" AND (name ILIKE ${} OR email ILIKE ${})", param_count, param_count + 1));
        count_query.push_str(&format!(" AND (name ILIKE ${} OR email ILIKE ${})", param_count, param_count + 1));
        params.push(search_param.clone());
        params.push(search_param);
        param_count += 1;
    }
    
    if let Some(skills) = &filters.skills {
        for skill in skills {
            param_count += 1;
            base_query.push_str(&format!(" AND ${} = ANY(skills)", param_count));
            count_query.push_str(&format!(" AND ${} = ANY(skills)", param_count));
            params.push(skill.clone());
        }
    }
    
    if let Some(min_exp) = filters.min_experience {
        param_count += 1;
        base_query.push_str(&format!(" AND experience_years >= ${}", param_count));
        count_query.push_str(&format!(" AND experience_years >= ${}", param_count));
        params.push(min_exp.to_string());
    }
    
    if let Some(max_exp) = filters.max_experience {
        param_count += 1;
        base_query.push_str(&format!(" AND experience_years <= ${}", param_count));
        count_query.push_str(&format!(" AND experience_years <= ${}", param_count));
        params.push(max_exp.to_string());
    }
    
    base_query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));
    
    // Execute count query
    let total: i64 = sqlx::query_scalar(&count_query)
        .bind(&params.get(0).unwrap_or(&String::new()))
        .bind(&params.get(1).unwrap_or(&String::new()))
        .fetch_one(&pool)
        .await?;
    
    // Execute main query
    let mut query = sqlx::query_as::<_, Worker>(&base_query);
    for param in &params {
        query = query.bind(param);
    }
    query = query.bind(pagination.limit()).bind(pagination.offset());
    
    let workers = query.fetch_all(&pool).await?;
    
    let response = PaginatedResponse::new(workers, pagination.page.unwrap_or(1), pagination.limit.unwrap_or(20), total);
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_worker(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let worker = sqlx::query_as!(
        Worker,
        r#"
        SELECT id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        FROM workers
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?;
    
    match worker {
        Some(worker) => Ok(Json(ApiResponse::success(worker))),
        None => Err(AppError::NotFound),
    }
}

pub async fn create_worker(
    State(pool): State<PgPool>,
    Json(mut request): Json<CreateWorkerRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest("Worker name is required".to_string()));
    }
    
    if !validate_email(&request.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    
    if let Some(phone) = &request.phone {
        if !validate_phone(phone) {
            return Err(AppError::BadRequest("Invalid phone format".to_string()));
        }
    }
    
    if request.experience_years < 0 {
        return Err(AppError::BadRequest("Experience years cannot be negative".to_string()));
    }
    
    // Check if email already exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workers WHERE email = $1",
        request.email
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Email already exists".to_string()));
    }
    
    // Clean and deduplicate skills
    request.skills.sort();
    request.skills.dedup();
    
    let worker = sqlx::query_as!(
        Worker,
        r#"
        INSERT INTO workers (id, name, email, phone, skills, experience_years, resume_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        "#,
        Uuid::new_v4(),
        request.name.trim(),
        request.email.trim().to_lowercase(),
        request.phone,
        &request.skills,
        request.experience_years,
        request.resume_url
    )
    .fetch_one(&pool)
    .await?;
    
    Ok((StatusCode::CREATED, Json(ApiResponse::success(worker))))
}

pub async fn update_worker(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(mut request): Json<CreateWorkerRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.name.trim().is_empty() {
        return Err(AppError::BadRequest("Worker name is required".to_string()));
    }
    
    if !validate_email(&request.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    
    if let Some(phone) = &request.phone {
        if !validate_phone(phone) {
            return Err(AppError::BadRequest("Invalid phone format".to_string()));
        }
    }
    
    if request.experience_years < 0 {
        return Err(AppError::BadRequest("Experience years cannot be negative".to_string()));
    }
    
    // Check if worker exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workers WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    // Check if email already exists for another worker
    let email_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workers WHERE email = $1 AND id != $2",
        request.email,
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if email_exists.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Email already exists".to_string()));
    }
    
    // Clean and deduplicate skills
    request.skills.sort();
    request.skills.dedup();
    
    let worker = sqlx::query_as!(
        Worker,
        r#"
        UPDATE workers 
        SET name = $1, email = $2, phone = $3, skills = $4, experience_years = $5, resume_url = $6, updated_at = NOW()
        WHERE id = $7
        RETURNING id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        "#,
        request.name.trim(),
        request.email.trim().to_lowercase(),
        request.phone,
        &request.skills,
        request.experience_years,
        request.resume_url,
        id
    )
    .fetch_one(&pool)
    .await?;
    
    Ok(Json(ApiResponse::success(worker)))
}

pub async fn delete_worker(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check if worker exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workers WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    // Check if worker has active applications
    let active_applications = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM applications WHERE worker_id = $1 AND status IN ('pending', 'reviewing')",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if active_applications.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Cannot delete worker with active applications".to_string()));
    }
    
    let result = sqlx::query!(
        "DELETE FROM workers WHERE id = $1",
        id
    )
    .execute(&pool)
    .await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": true}))))
}

pub async fn get_worker_skills(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let skills = sqlx::query_scalar!(
        "SELECT DISTINCT unnest(skills) as skill FROM workers WHERE skills IS NOT NULL AND array_length(skills, 1) > 0"
    )
    .fetch_all(&pool)
    .await?;
    
    let mut unique_skills = std::collections::HashSet::new();
    for skill in skills {
        if let Some(skill) = skill {
            unique_skills.insert(skill);
        }
    }
    
    let mut sorted_skills: Vec<String> = unique_skills.into_iter().collect();
    sorted_skills.sort();
    
    Ok(Json(ApiResponse::success(sorted_skills)))
}
