use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashSet;

use crate::models::*;
use crate::error::AppError;
use crate::auth::{AuthService, CurrentUser};

// Query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct JobFilterQuery {
    pub client_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub location: Option<String>,
    pub job_type: Option<String>,
}

// Client handlers
pub async fn get_clients(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let clients = sqlx::query_as!(
        Client,
        r#"
        SELECT id, company_name, email, phone, created_at, updated_at
        FROM clients
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(clients))
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
    .fetch_one(&pool)
    .await?;

    Ok(Json(client))
}

pub async fn create_client(
    State(pool): State<PgPool>,
    Json(request): Json<CreateClientRequest>,
) -> Result<impl IntoResponse, AppError> {
    let client = sqlx::query_as!(
        Client,
        r#"
        INSERT INTO clients (id, company_name, email, phone)
        VALUES ($1, $2, $3, $4)
        RETURNING id, company_name, email, phone, created_at, updated_at
        "#,
        Uuid::new_v4(),
        request.company_name,
        request.email,
        request.phone
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(client)))
}

// Worker handlers
pub async fn get_workers(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let workers = sqlx::query_as!(
        Worker,
        r#"
        SELECT id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        FROM workers
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(workers))
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
    .fetch_one(&pool)
    .await?;

    Ok(Json(worker))
}

pub async fn create_worker(
    State(pool): State<PgPool>,
    Json(request): Json<CreateWorkerRequest>,
) -> Result<impl IntoResponse, AppError> {
    let worker = sqlx::query_as!(
        Worker,
        r#"
        INSERT INTO workers (id, name, email, phone, skills, experience_years, resume_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        "#,
        Uuid::new_v4(),
        request.name,
        request.email,
        request.phone,
        &request.skills,
        request.experience_years,
        request.resume_url
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(worker)))
}

// Job handlers
pub async fn get_jobs(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationQuery>,
    Query(filters): Query<JobFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let mut query = "
        SELECT j.id, j.client_id, j.title, j.description, j.requirements, 
               j.salary_range, j.location, j.job_type, j.is_active, j.created_at, j.updated_at
        FROM job_postings j
        WHERE 1=1
    ".to_string();

    let mut params = Vec::new();
    let mut param_count = 0;

    if let Some(client_id) = filters.client_id {
        param_count += 1;
        query.push_str(&format!(" AND j.client_id = ${}", param_count));
        params.push(client_id);
    }

    if let Some(is_active) = filters.is_active {
        param_count += 1;
        query.push_str(&format!(" AND j.is_active = ${}", param_count));
        params.push(is_active);
    }

    if let Some(location) = filters.location {
        param_count += 1;
        query.push_str(&format!(" AND j.location ILIKE ${}", param_count));
        params.push(format!("%{}%", location));
    }

    if let Some(job_type) = filters.job_type {
        param_count += 1;
        query.push_str(&format!(" AND j.job_type = ${}", param_count));
        params.push(job_type);
    }

    query.push_str(" ORDER BY j.created_at DESC");
    param_count += 1;
    query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_count, param_count + 1));
    params.push(limit as i64);
    params.push(offset as i64);

    let mut query_builder = sqlx::query_as::<_, JobPosting>(&query);
    for param in params {
        if let Some(client_id) = param.downcast_ref::<Uuid>() {
            query_builder = query_builder.bind(client_id);
        } else if let Some(is_active) = param.downcast_ref::<bool>() {
            query_builder = query_builder.bind(is_active);
        } else if let Some(location) = param.downcast_ref::<String>() {
            query_builder = query_builder.bind(location);
        }
    }

    let jobs = query_builder.fetch_all(&pool).await?;

    Ok(Json(jobs))
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
    .fetch_one(&pool)
    .await?;

    Ok(Json(job))
}

pub async fn create_job(
    State(pool): State<PgPool>,
    _current_user: CurrentUser,
    Json(request): Json<CreateJobRequest>,
) -> Result<impl IntoResponse, AppError> {
    let job = sqlx::query_as!(
        JobPosting,
        r#"
        INSERT INTO job_postings (id, client_id, title, description, requirements, salary_range, location, job_type, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, client_id, title, description, requirements, salary_range, location, job_type, is_active, created_at, updated_at
        "#,
        Uuid::new_v4(),
        request.client_id,
        request.title,
        request.description,
        &request.requirements,
        request.salary_range,
        request.location,
        request.job_type.unwrap_or_else(|| "full-time".to_string()),
        request.is_active.unwrap_or(true)
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(job)))
}

// Meeting handlers
pub async fn get_meetings(
    State(pool): State<PgPool>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    let meetings = sqlx::query_as!(
        Meeting,
        r#"
        SELECT id, client_id, worker_id, job_id, title, description, scheduled_at, 
               duration_minutes, status, meeting_url, location, created_at, updated_at
        FROM meetings
        ORDER BY scheduled_at ASC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(meetings))
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
    .fetch_one(&pool)
    .await?;

    Ok(Json(meeting))
}

pub async fn create_meeting(
    State(pool): State<PgPool>,
    _current_user: CurrentUser,
    Json(request): Json<CreateMeetingRequest>,
) -> Result<impl IntoResponse, AppError> {
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
        request.title,
        request.description,
        request.scheduled_at,
        request.duration_minutes.unwrap_or(60),
        "scheduled".to_string(),
        request.meeting_url,
        request.location
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(meeting)))
}

// Job matching algorithm
pub async fn find_matches(
    State(pool): State<PgPool>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let job = sqlx::query_as!(
        JobPosting,
        r#"
        SELECT id, client_id, title, description, requirements, salary_range, location, job_type, is_active, created_at, updated_at
        FROM job_postings
        WHERE id = $1 AND is_active = true
        "#,
        job_id
    )
    .fetch_one(&pool)
    .await?;

    let workers = sqlx::query_as!(
        Worker,
        r#"
        SELECT id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        FROM workers
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let job_requirements: HashSet<String> = job.requirements.iter().cloned().collect();
    
    let mut matched_workers = Vec::new();
    let mut match_scores = Vec::new();

    for worker in workers {
        let worker_skills: HashSet<String> = worker.skills.iter().cloned().collect();
        
        // Calculate matching skills
        let matching_skills: Vec<String> = worker_skills
            .intersection(&job_requirements)
            .cloned()
            .collect();

        if !matching_skills.is_empty() {
            // Calculate match score (0-100)
            let skill_score = (matching_skills.len() as f64 / job_requirements.len().max(1) as f64) * 70.0;
            let experience_bonus = (worker.experience_years.min(10) as f64 / 10.0) * 30.0;
            let total_score = skill_score + experience_bonus;

            matched_workers.push(worker.clone());
            match_scores.push(WorkerMatchScore {
                worker,
                score: total_score,
                matching_skills,
            });
        }
    }

    // Sort by score descending
    match_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let matched_workers_only: Vec<Worker> = match_scores
        .iter()
        .map(|ms| ms.worker.clone())
        .collect();

    let response = JobMatchResponse {
        job,
        matched_workers: matched_workers_only,
        match_count: matched_workers_only.len(),
        match_scores,
    };

    Ok(Json(response))
}

// Auth handlers
pub async fn register(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_service.register_user(&pool, request).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = auth_service.authenticate_user(&pool, request).await?;
    Ok(Json(response))
}

pub async fn get_current_user(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    CurrentUser(current_user): CurrentUser,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_service.get_user_by_id(&pool, current_user.id).await?;
    Ok(Json(user))
}
