use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::models::{User, LoginRequest, LoginResponse, CreateUserRequest};
use crate::error::AppError;
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub role: String,
    pub exp: i64, // expiration time
}

pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AppError> {
        hash(password, DEFAULT_COST)
            .map_err(|e| AppError::InternalServerError(format!("Failed to hash password: {}", e)))
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        verify(password, hash)
            .map_err(|e| AppError::InternalServerError(format!("Failed to verify password: {}", e)))
    }

    pub fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| AppError::InternalServerError(format!("Failed to generate token: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        Ok(token_data.claims)
    }

    pub async fn register_user(
        &self,
        pool: &PgPool,
        request: CreateUserRequest,
    ) -> Result<User, AppError> {
        // Check if user already exists
        let existing_user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            request.email
        )
        .fetch_optional(pool)
        .await?;

        if existing_user.is_some() {
            return Err(AppError::BadRequest("User with this email already exists".to_string()));
        }

        let password_hash = self.hash_password(&request.password)?;
        let user_id = Uuid::new_v4();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, email, password_hash, role, client_id, worker_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, email, password_hash, role, client_id, worker_id, is_active, created_at, updated_at
            "#,
            user_id,
            request.email,
            password_hash,
            request.role,
            request.client_id,
            request.worker_id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn authenticate_user(
        &self,
        pool: &PgPool,
        request: LoginRequest,
    ) -> Result<LoginResponse, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1 AND is_active = true",
            request.email
        )
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

        let is_valid = self.verify_password(&request.password, &user.password_hash)?;

        if !is_valid {
            return Err(AppError::Unauthorized);
        }

        let token = self.generate_token(&user)?;

        Ok(LoginResponse { token, user })
    }

    pub async fn get_user_by_id(&self, pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1 AND is_active = true",
            user_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

        Ok(user)
    }
}

// Middleware for JWT authentication
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = auth_service
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user info to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// Extractor for getting current user from request
use axum::extract::FromRequestParts;
use http::request::Parts;

pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let id = Uuid::parse_str(&claims.sub)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(CurrentUser {
            id,
            email: claims.email.clone(),
            role: claims.role.clone(),
        })
    }
}
