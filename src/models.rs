use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Client {
    pub id: Uuid,
    pub company_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub skills: Vec<String>,
    pub experience_years: i32,
    pub resume_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct JobPosting {
    pub id: Uuid,
    pub client_id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub salary_range: Option<String>,
    pub location: String,
    pub job_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: Uuid,
    pub job_id: Uuid,
    pub worker_id: Uuid,
    pub status: String,
    pub cover_letter: Option<String>,
    pub applied_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Meeting {
    pub id: Uuid,
    pub client_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub job_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: i32,
    pub status: String,
    pub meeting_url: Option<String>,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub client_id: Option<Uuid>,
    pub worker_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
    pub company_name: String,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkerRequest {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub skills: Vec<String>,
    pub experience_years: i32,
    pub resume_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub client_id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub salary_range: Option<String>,
    pub location: String,
    pub job_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMeetingRequest {
    pub client_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub job_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: Option<i32>,
    pub meeting_url: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub role: String,
    pub client_id: Option<Uuid>,
    pub worker_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct JobMatchResponse {
    pub job: JobPosting,
    pub matched_workers: Vec<Worker>,
    pub match_count: usize,
    pub match_scores: Vec<WorkerMatchScore>,
}

#[derive(Debug, Serialize)]
pub struct WorkerMatchScore {
    pub worker: Worker,
    pub score: f64,
    pub matching_skills: Vec<String>,
}
