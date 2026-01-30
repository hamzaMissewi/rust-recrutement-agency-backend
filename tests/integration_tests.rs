use reqwest;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_health_check() {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/")
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_user_registration_and_login() {
    let client = reqwest::Client::new();
    
    // Register a new user
    let register_payload = json!({
        "email": "test@example.com",
        "password": "password123",
        "role": "admin"
    });
    
    let register_response = client
        .post("http://localhost:3000/api/auth/register")
        .json(&register_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(register_response.status(), 201);
    
    // Login with the new user
    let login_payload = json!({
        "email": "test@example.com",
        "password": "password123"
    });
    
    let login_response = client
        .post("http://localhost:3000/api/auth/login")
        .json(&login_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(login_response.status(), 200);
    
    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse JSON");
    assert!(login_body["data"]["token"].is_string());
}

#[tokio::test]
async fn test_create_client() {
    let client = reqwest::Client::new();
    
    // First login to get token
    let login_payload = json!({
        "email": "test@example.com",
        "password": "password123"
    });
    
    let login_response = client
        .post("http://localhost:3000/api/auth/login")
        .json(&login_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse JSON");
    let token = login_body["data"]["token"].as_str().expect("No token found");
    
    // Create a client
    let client_payload = json!({
        "company_name": "Test Company",
        "email": "contact@testcompany.com",
        "phone": "+1-555-0123"
    });
    
    let create_response = client
        .post("http://localhost:3000/api/clients")
        .header("Authorization", format!("Bearer {}", token))
        .json(&client_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(create_response.status(), 201);
    
    let create_body: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    assert_eq!(create_body["data"]["company_name"], "Test Company");
}

#[tokio::test]
async fn test_create_worker() {
    let client = reqwest::Client::new();
    
    // Login to get token
    let login_payload = json!({
        "email": "test@example.com",
        "password": "password123"
    });
    
    let login_response = client
        .post("http://localhost:3000/api/auth/login")
        .json(&login_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse JSON");
    let token = login_body["data"]["token"].as_str().expect("No token found");
    
    // Create a worker
    let worker_payload = json!({
        "name": "John Doe",
        "email": "john.doe@example.com",
        "phone": "+1-555-0456",
        "skills": ["Rust", "PostgreSQL", "Docker"],
        "experience_years": 5
    });
    
    let create_response = client
        .post("http://localhost:3000/api/workers")
        .header("Authorization", format!("Bearer {}", token))
        .json(&worker_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(create_response.status(), 201);
    
    let create_body: serde_json::Value = create_response.json().await.expect("Failed to parse JSON");
    assert_eq!(create_body["data"]["name"], "John Doe");
}

#[tokio::test]
async fn test_job_matching() {
    let client = reqwest::Client::new();
    
    // Login to get token
    let login_payload = json!({
        "email": "test@example.com",
        "password": "password123"
    });
    
    let login_response = client
        .post("http://localhost:3000/api/auth/login")
        .json(&login_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let login_body: serde_json::Value = login_response.json().await.expect("Failed to parse JSON");
    let token = login_body["data"]["token"].as_str().expect("No token found");
    
    // Create a job posting
    let job_payload = json!({
        "client_id": "00000000-0000-0000-0000-000000000000", // This would need to be a real client ID
        "title": "Rust Developer",
        "description": "Looking for an experienced Rust developer",
        "requirements": ["Rust", "PostgreSQL", "Docker"],
        "location": "Remote",
        "job_type": "full-time"
    });
    
    let job_response = client
        .post("http://localhost:3000/api/jobs")
        .header("Authorization", format!("Bearer {}", token))
        .json(&job_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Note: This test would need a real client ID to work properly
    // In a real test environment, you would first create a client and use its ID
}
