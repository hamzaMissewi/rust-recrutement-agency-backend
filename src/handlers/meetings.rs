use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::*;
use crate::error::AppError;
use crate::utils::{ApiResponse, PaginationParams, PaginatedResponse};

#[derive(Debug, Deserialize)]
pub struct MeetingFilterQuery {
    pub client_id: Option<Uuid>,
    pub worker_id: Option<Uuid>,
    pub status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

pub async fn get_meetings(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<MeetingFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let pagination = PaginationParams::new(pagination.page, pagination.limit);
    
    let mut base_query = "
        SELECT m.id, m.client_id, m.worker_id, m.job_id, m.title, m.description, m.scheduled_at, 
               m.duration_minutes, m.status, m.meeting_url, m.location, m.created_at, m.updated_at,
               c.company_name as client_name,
               w.name as worker_name
        FROM meetings m
        JOIN clients c ON m.client_id = c.id
        LEFT JOIN workers w ON m.worker_id = w.id
        WHERE 1=1
    ".to_string();
    
    let mut count_query = "
        SELECT COUNT(*) as total FROM meetings m WHERE 1=1
    ".to_string();
    
    let mut params = Vec::new();
    let mut param_count = 0;
    
    if let Some(client_id) = filters.client_id {
        param_count += 1;
        base_query.push_str(&format!(" AND m.client_id = ${}", param_count));
        count_query.push_str(&format!(" AND m.client_id = ${}", param_count));
        params.push(client_id.to_string());
    }
    
    if let Some(worker_id) = filters.worker_id {
        param_count += 1;
        base_query.push_str(&format!(" AND m.worker_id = ${}", param_count));
        count_query.push_str(&format!(" AND m.worker_id = ${}", param_count));
        params.push(worker_id.to_string());
    }
    
    if let Some(status) = &filters.status {
        param_count += 1;
        base_query.push_str(&format!(" AND m.status = ${}", param_count));
        count_query.push_str(&format!(" AND m.status = ${}", param_count));
        params.push(status.clone());
    }
    
    if let Some(from_date) = filters.from_date {
        param_count += 1;
        base_query.push_str(&format!(" AND m.scheduled_at >= ${}", param_count));
        count_query.push_str(&format!(" AND m.scheduled_at >= ${}", param_count));
        params.push(from_date.to_rfc3339());
    }
    
    if let Some(to_date) = filters.to_date {
        param_count += 1;
        base_query.push_str(&format!(" AND m.scheduled_at <= ${}", param_count));
        count_query.push_str(&format!(" AND m.scheduled_at <= ${}", param_count));
        params.push(to_date.to_rfc3339());
    }
    
    base_query.push_str(&format!(" ORDER BY m.scheduled_at ASC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));
    
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
    
    let mut meetings = Vec::new();
    for row in rows {
        let meeting = Meeting {
            id: row.get("id"),
            client_id: row.get("client_id"),
            worker_id: row.get("worker_id"),
            job_id: row.get("job_id"),
            title: row.get("title"),
            description: row.get("description"),
            scheduled_at: row.get("scheduled_at"),
            duration_minutes: row.get("duration_minutes"),
            status: row.get("status"),
            meeting_url: row.get("meeting_url"),
            location: row.get("location"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        meetings.push(meeting);
    }
    
    let response = PaginatedResponse::new(meetings, pagination.page.unwrap_or(1), pagination.limit.unwrap_or(20), total);
    
    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_meeting(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let meeting = sqlx::query_as!(
        Meeting,
        r#"
        SELECT id, client_id, worker_id, job_id, title, description, scheduled_at, 
               duration_minutes, status, meeting_url, location, created_at, updated_at
        FROM meetings
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?;
    
    match meeting {
        Some(meeting) => Ok(Json(ApiResponse::success(meeting))),
        None => Err(AppError::NotFound),
    }
}

pub async fn create_meeting(
    State(pool): State<PgPool>,
    Json(mut request): Json<CreateMeetingRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.title.trim().is_empty() {
        return Err(AppError::BadRequest("Meeting title is required".to_string()));
    }
    
    if request.scheduled_at <= Utc::now() {
        return Err(AppError::BadRequest("Meeting must be scheduled in the future".to_string()));
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
    
    // Check if worker exists (if provided)
    if let Some(worker_id) = request.worker_id {
        let worker_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workers WHERE id = $1",
            worker_id
        )
        .fetch_one(&pool)
        .await?;
        
        if worker_exists.unwrap_or(0) == 0 {
            return Err(AppError::BadRequest("Worker not found".to_string()));
        }
    }
    
    // Check if job exists (if provided)
    if let Some(job_id) = request.job_id {
        let job_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM job_postings WHERE id = $1",
            job_id
        )
        .fetch_one(&pool)
        .await?;
        
        if job_exists.unwrap_or(0) == 0 {
            return Err(AppError::BadRequest("Job not found".to_string()));
        }
    }
    
    let meeting = sqlx::query_as!(
        Meeting,
        r#"
        INSERT INTO meetings (id, client_id, worker_id, job_id, title, description, scheduled_at, duration_minutes, status, meeting_url, location)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, client_id, worker_id, job_id, title, description, scheduled_at, 
                  duration_minutes, status, meeting_url, location, created_at, updated_at
        "#,
        Uuid::new_v4(),
        request.client_id,
        request.worker_id,
        request.job_id,
        request.title.trim(),
        request.description,
        request.scheduled_at,
        request.duration_minutes.unwrap_or(60),
        "scheduled".to_string(),
        request.meeting_url,
        request.location
    )
    .fetch_one(&pool)
    .await?;
    
    Ok((StatusCode::CREATED, Json(ApiResponse::success(meeting))))
}

pub async fn update_meeting(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(mut request): Json<CreateMeetingRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.title.trim().is_empty() {
        return Err(AppError::BadRequest("Meeting title is required".to_string()));
    }
    
    // Check if meeting exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM meetings WHERE id = $1",
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
    
    // Check if worker exists (if provided)
    if let Some(worker_id) = request.worker_id {
        let worker_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workers WHERE id = $1",
            worker_id
        )
        .fetch_one(&pool)
        .await?;
        
        if worker_exists.unwrap_or(0) == 0 {
            return Err(AppError::BadRequest("Worker not found".to_string()));
        }
    }
    
    let meeting = sqlx::query_as!(
        Meeting,
        r#"
        UPDATE meetings 
        SET client_id = $1, worker_id = $2, job_id = $3, title = $4, description = $5, 
            scheduled_at = $6, duration_minutes = $7, meeting_url = $8, location = $9, updated_at = NOW()
        WHERE id = $10
        RETURNING id, client_id, worker_id, job_id, title, description, scheduled_at, 
                  duration_minutes, status, meeting_url, location, created_at, updated_at
        "#,
        request.client_id,
        request.worker_id,
        request.job_id,
        request.title.trim(),
        request.description,
        request.scheduled_at,
        request.duration_minutes.unwrap_or(60),
        request.meeting_url,
        request.location,
        id
    )
    .fetch_one(&pool)
    .await?;
    
    Ok(Json(ApiResponse::success(meeting)))
}

pub async fn update_meeting_status(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let status = request.get("status")
        .and_then(|s| s.as_str())
        .ok_or(AppError::BadRequest("Status is required".to_string()))?;
    
    let valid_statuses = ["scheduled", "completed", "cancelled", "rescheduled"];
    if !valid_statuses.contains(&status) {
        return Err(AppError::BadRequest("Invalid status".to_string()));
    }
    
    // Check if meeting exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM meetings WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    let meeting = sqlx::query_as!(
        Meeting,
        r#"
        UPDATE meetings 
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, client_id, worker_id, job_id, title, description, scheduled_at, 
                  duration_minutes, status, meeting_url, location, created_at, updated_at
        "#,
        status,
        id
    )
    .fetch_one(&pool)
    .await?;
    
    Ok(Json(ApiResponse::success(meeting)))
}

pub async fn delete_meeting(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check if meeting exists
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM meetings WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    
    if existing.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }
    
    let result = sqlx::query!(
        "DELETE FROM meetings WHERE id = $1",
        id
    )
    .execute(&pool)
    .await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(Json(ApiResponse::success(serde_json::json!({"deleted": true}))))
}

pub async fn get_upcoming_meetings(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let meetings = sqlx::query_as!(
        Meeting,
        r#"
        SELECT id, client_id, worker_id, job_id, title, description, scheduled_at, 
               duration_minutes, status, meeting_url, location, created_at, updated_at
        FROM meetings
        WHERE scheduled_at > NOW() AND status = 'scheduled'
        ORDER BY scheduled_at ASC
        LIMIT 10
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    Ok(Json(ApiResponse::success(meetings)))
}
