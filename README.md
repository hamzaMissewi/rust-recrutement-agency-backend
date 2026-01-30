# Recruitment Agency Backend

A Rust-based REST API for a recruitment agency application built with Axum, SQLx, and PostgreSQL.

## Features

- **Authentication**: JWT-based authentication with role-based access control
- **Client Management**: Manage recruitment companies and their information
- **Worker Profiles**: Store and manage candidate profiles with skills and experience
- **Job Postings**: Create and manage job opportunities with requirements
- **Smart Matching**: AI-powered job-candidate matching algorithm
- **Meeting Scheduling**: Schedule and manage meetings between clients and candidates
- **RESTful API**: Clean, well-documented API endpoints
- **Database**: PostgreSQL with automatic migrations

## Tech Stack

- **Rust** - Programming language
- **Axum** - Web framework
- **SQLx** - Database toolkit
- **PostgreSQL** - Database
- **JWT** - Authentication tokens
- **bcrypt** - Password hashing
- **Tower** - Middleware
- **Serde** - Serialization/Deserialization

## Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- Docker (optional)

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd recruitment-backend
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   ```
   
   Edit `.env` with your configuration:
   ```env
   DATABASE_URL=postgres://postgres:password@localhost/recruitment
   JWT_SECRET=your-super-secret-jwt-key-here
   RUST_LOG=info
   ```

3. **Set up the database**
   
   **Option 1: Using Docker**
   ```bash
   docker run --name recruitment-db -e POSTGRES_PASSWORD=password -e POSTGRES_DB=recruitment -p 5432:5432 -d postgres:15
   ```
   
   **Option 2: Local PostgreSQL**
   ```sql
   CREATE DATABASE recruitment;
   ```

4. **Install dependencies and run**
   ```bash
   cargo build
   cargo run
   ```

The server will start on `http://localhost:3000`

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login user
- `GET /api/auth/me` - Get current user (protected)

### Clients
- `GET /api/clients` - Get all clients
- `POST /api/clients` - Create new client
- `GET /api/clients/:id` - Get specific client

### Workers
- `GET /api/workers` - Get all workers
- `POST /api/workers` - Create new worker
- `GET /api/workers/:id` - Get specific worker

### Jobs
- `GET /api/jobs` - Get all jobs (with filtering)
- `POST /api/jobs` - Create new job (protected)
- `GET /api/jobs/:id` - Get specific job

### Meetings
- `GET /api/meetings` - Get all meetings
- `POST /api/meetings` - Create new meeting (protected)
- `GET /api/meetings/:id` - Get specific meeting

### Matching
- `GET /api/match/:job_id` - Find matching candidates for a job

### Health Check
- `GET /` - Health check endpoint

## API Usage Examples

### Register a new user
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "password123",
    "role": "admin"
  }'
```

### Login
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "password123"
  }'
```

### Create a job posting
```bash
curl -X POST http://localhost:3000/api/jobs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "client_id": "client-uuid",
    "title": "Senior Frontend Developer",
    "description": "We are looking for an experienced frontend developer...",
    "requirements": ["JavaScript", "React", "TypeScript"],
    "salary_range": "$80,000 - $120,000",
    "location": "Remote"
  }'
```

### Find matching candidates
```bash
curl -X GET http://localhost:3000/api/match/job-uuid \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Database Schema

The application uses the following main tables:

- **clients** - Recruitment companies
- **workers** - Job candidates
- **job_postings** - Job opportunities
- **applications** - Job applications
- **meetings** - Scheduled meetings
- **users** - Authentication users

## Matching Algorithm

The matching algorithm considers:

1. **Skill Matching** (70% weight): Overlap between job requirements and worker skills
2. **Experience** (30% weight): Years of relevant experience
3. **Location Preference**: Optional location matching
4. **Score Calculation**: Final score 0-100, ranked by relevance

## Development

### Running tests
```bash
cargo test
```

### Development mode with auto-reload
```bash
cargo install cargo-watch
cargo watch -x run
```

### Database migrations
The application automatically runs migrations on startup. The migration files are located in the `database.rs` module.

## Production Deployment

### Environment variables
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret key for JWT tokens
- `RUST_LOG` - Log level (info, debug, warn, error)

### Docker deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/recruitment-backend /usr/local/bin/recruitment-backend
EXPOSE 3000
CMD ["recruitment-backend"]
```

## Security

- Passwords are hashed using bcrypt
- JWT tokens expire after 7 days
- CORS is configured for cross-origin requests
- Input validation on all endpoints
- SQL injection protection via SQLx


## License
Hamza Missaoui
