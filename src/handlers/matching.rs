use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::error::AppError;
use crate::utils::{ApiResponse, calculate_skill_match_score, calculate_experience_score};

#[derive(Debug, Deserialize)]
pub struct MatchQuery {
    pub min_score: Option<f64>,
    pub limit: Option<u32>,
}

pub async fn find_matches(
    State(pool): State<PgPool>,
    Path(job_id): Path<Uuid>,
    Query(query): Query<MatchQuery>,
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
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

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

    let min_score = query.min_score.unwrap_or(0.0);
    let limit = query.limit.unwrap_or(50).min(100);

    let mut match_scores = Vec::new();

    for worker in workers {
        let skill_score = calculate_skill_match_score(&job.requirements, &worker.skills);
        let experience_score = calculate_experience_score(worker.experience_years, 3); // Assume 3 years minimum
        let total_score = (skill_score * 0.7) + (experience_score * 0.3);

        if total_score >= min_score {
            let matching_skills: Vec<String> = worker.skills
                .iter()
                .filter(|skill| job.requirements.contains(skill))
                .cloned()
                .collect();

            match_scores.push(WorkerMatchScore {
                worker,
                score: total_score,
                matching_skills,
            });
        }
    }

    // Sort by score descending
    match_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Limit results
    match_scores.truncate(limit as usize);

    let matched_workers: Vec<Worker> = match_scores
        .iter()
        .map(|ms| ms.worker.clone())
        .collect();

    let response = JobMatchResponse {
        job,
        matched_workers: matched_workers.clone(),
        match_count: matched_workers.len(),
        match_scores,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn find_jobs_for_worker(
    State(pool): State<PgPool>,
    Path(worker_id): Path<Uuid>,
    Query(query): Query<MatchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let worker = sqlx::query_as!(
        Worker,
        r#"
        SELECT id, name, email, phone, skills, experience_years, resume_url, created_at, updated_at
        FROM workers
        WHERE id = $1
        "#,
        worker_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let jobs = sqlx::query_as!(
        JobPosting,
        r#"
        SELECT id, client_id, title, description, requirements, salary_range, location, job_type, is_active, created_at, updated_at
        FROM job_postings
        WHERE is_active = true
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let min_score = query.min_score.unwrap_or(0.0);
    let limit = query.limit.unwrap_or(50).min(100);

    let mut job_matches = Vec::new();

    for job in jobs {
        let skill_score = calculate_skill_match_score(&job.requirements, &worker.skills);
        let experience_score = calculate_experience_score(worker.experience_years, 3); // Assume 3 years minimum
        let total_score = (skill_score * 0.7) + (experience_score * 0.3);

        if total_score >= min_score {
            let matching_requirements: Vec<String> = job.requirements
                .iter()
                .filter(|req| worker.skills.contains(req))
                .cloned()
                .collect();

            job_matches.push(JobMatchScore {
                job,
                score: total_score,
                matching_skills: matching_requirements,
            });
        }
    }

    // Sort by score descending
    job_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Limit results
    job_matches.truncate(limit as usize);

    let matched_jobs: Vec<JobPosting> = job_matches
        .iter()
        .map(|jm| jm.job.clone())
        .collect();

    let response = serde_json::json!({
        "worker": worker,
        "matched_jobs": matched_jobs,
        "match_count": matched_jobs.len(),
        "job_matches": job_matches
    });

    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, serde::Serialize)]
pub struct JobMatchScore {
    pub job: JobPosting,
    pub score: f64,
    pub matching_skills: Vec<String>,
}

pub async fn get_matching_stats(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let total_jobs: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM job_postings WHERE is_active = true"
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0);

    let total_workers: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workers"
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0);

    let avg_requirements: f64 = sqlx::query_scalar!(
        "SELECT AVG(array_length(requirements, 1)) FROM job_postings WHERE is_active = true AND requirements IS NOT NULL"
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0.0);

    let avg_skills: f64 = sqlx::query_scalar!(
        "SELECT AVG(array_length(skills, 1)) FROM workers WHERE skills IS NOT NULL"
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0.0);

    let response = serde_json::json!({
        "total_active_jobs": total_jobs,
        "total_workers": total_workers,
        "average_requirements_per_job": avg_requirements,
        "average_skills_per_worker": avg_skills,
        "potential_matches": total_jobs * total_workers
    });

    Ok(Json(ApiResponse::success(response)))
}
