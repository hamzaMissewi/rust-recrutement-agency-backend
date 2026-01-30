use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::auth::{AuthService, CurrentUser};
use crate::models::*;
use crate::error::AppError;
use crate::utils::{ApiResponse, validate_email};
use sqlx::PgPool;
use chrono;

pub async fn register(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }

    if !validate_email(&request.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    if request.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
    }

    // Password complexity validation
    if !is_strong_password(&request.password) {
        return Err(AppError::BadRequest(
            "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character".to_string()
        ));
    }

    let valid_roles = ["admin", "client", "worker"];
    if !valid_roles.contains(&request.role.as_str()) {
        return Err(AppError::BadRequest("Invalid role. Must be one of: admin, client, worker".to_string()));
    }

    // Additional validation for role-specific requirements
    match request.role.as_str() {
        "client" => {
            if request.client_id.is_none() {
                return Err(AppError::BadRequest("Client ID is required for client role".to_string()));
            }
        },
        "worker" => {
            if request.worker_id.is_none() {
                return Err(AppError::BadRequest("Worker ID is required for worker role".to_string()));
            }
        },
        _ => {} // Admin doesn't need specific IDs
    }

    let user = auth_service.register_user(&pool, request).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::success(user))))
}

pub async fn login(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if request.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }

    if !validate_email(&request.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    if request.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    // Rate limiting could be added here
    let response = auth_service.authenticate_user(&pool, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_current_user(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    CurrentUser(current_user): CurrentUser,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_service.get_user_by_id(&pool, current_user.id).await?;
    
    // Remove sensitive information from response
    let safe_user = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "role": user.role,
        "client_id": user.client_id,
        "worker_id": user.worker_id,
        "is_active": user.is_active,
        "created_at": user.created_at,
        "updated_at": user.updated_at
    });
    
    Ok(Json(ApiResponse::success(safe_user)))
}

pub async fn update_password(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    CurrentUser(current_user): CurrentUser,
    Json(request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let current_password = request.get("current_password")
        .and_then(|p| p.as_str())
        .ok_or(AppError::BadRequest("Current password is required".to_string()))?;

    let new_password = request.get("new_password")
        .and_then(|p| p.as_str())
        .ok_or(AppError::BadRequest("New password is required".to_string()))?;

    let confirm_password = request.get("confirm_password")
        .and_then(|p| p.as_str())
        .ok_or(AppError::BadRequest("Password confirmation is required".to_string()))?;

    // Validate new password
    if new_password.len() < 8 {
        return Err(AppError::BadRequest("New password must be at least 8 characters".to_string()));
    }

    if !is_strong_password(new_password) {
        return Err(AppError::BadRequest(
            "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character".to_string()
        ));
    }

    if new_password != confirm_password {
        return Err(AppError::BadRequest("New password and confirmation do not match".to_string()));
    }

    // Prevent reusing the same password
    if current_password == new_password {
        return Err(AppError::BadRequest("New password must be different from current password".to_string()));
    }

    // Get current user with password
    let user = sqlx::query!(
        "SELECT password_hash FROM users WHERE id = $1 AND is_active = true",
        current_user.id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Verify current password
    let is_valid = auth_service.verify_password(current_password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::BadRequest("Current password is incorrect".to_string()));
    }

    // Hash new password
    let new_password_hash = auth_service.hash_password(new_password)?;

    // Update password
    sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
        new_password_hash,
        current_user.id
    )
    .execute(&pool)
    .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "Password updated successfully",
        "timestamp": chrono::Utc::now()
    }))))
}

pub async fn logout(
    State(_auth_service): State<AuthService>,
    CurrentUser(_current_user): CurrentUser,
) -> Result<impl IntoResponse, AppError> {
    // In a stateless JWT system, logout is typically handled client-side
    // by removing the token. For server-side logout, you could:
    // 1. Add the token to a blacklist
    // 2. Use short-lived tokens with refresh tokens
    // 3. Maintain a session store
    
    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "Logged out successfully",
        "instruction": "Please remove the token from client storage"
    }))))
}

pub async fn refresh_token(
    State(auth_service): State<AuthService>,
    State(pool): State<PgPool>,
    CurrentUser(current_user): CurrentUser,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_service.get_user_by_id(&pool, current_user.id).await?;
    
    // Generate new token
    let new_token = auth_service.generate_token(&user)?;
    
    Ok(Json(ApiResponse::success(serde_json::json!({
        "token": new_token,
        "user": {
            "id": user.id,
            "email": user.email,
            "role": user.role
        }
    }))))
}

// Helper function to validate password strength
fn is_strong_password(password: &str) -> bool {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    has_uppercase && has_lowercase && has_digit && has_special
}