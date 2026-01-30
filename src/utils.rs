use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Operation successful".to_string(),
            timestamp: Utc::now(),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl PaginationParams {
    pub fn new(page: Option<u32>, limit: Option<u32>) -> Self {
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(20).min(100); // Max 100 items per page
        let offset = (page - 1) * limit;
        
        Self { page: Some(page), limit: Some(limit), offset: Some(offset) }
    }
    
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20) as i64
    }
    
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as i64
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub limit: u32,
    pub total: i64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: u32, limit: u32, total: i64) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;
        
        Self {
            items,
            pagination: PaginationMeta {
                page,
                limit,
                total,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }
}

pub fn validate_email(email: &str) -> bool {
    // Simple email validation
    email.contains('@') && email.contains('.') && email.len() > 5
}

pub fn validate_phone(phone: &str) -> bool {
    // Basic phone validation - allows digits, spaces, +, -, (, )
    phone.chars().all(|c| c.is_ascii_digit() || " +-()".contains(c))
}

pub fn calculate_skill_match_score(job_skills: &[String], worker_skills: &[String]) -> f64 {
    if job_skills.is_empty() {
        return 0.0;
    }
    
    let job_skills_set: std::collections::HashSet<&str> = 
        job_skills.iter().map(|s| s.as_str()).collect();
    let worker_skills_set: std::collections::HashSet<&str> = 
        worker_skills.iter().map(|s| s.as_str()).collect();
    
    let matching_skills = job_skills_set.intersection(&worker_skills_set).count();
    (matching_skills as f64 / job_skills.len() as f64) * 100.0
}

pub fn calculate_experience_score(years_experience: i32, required_years: i32) -> f64 {
    if years_experience >= required_years {
        100.0
    } else {
        (years_experience as f64 / required_years.max(1) as f64) * 100.0
    }
}
