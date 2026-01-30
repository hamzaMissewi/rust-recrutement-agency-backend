# 🚀 Quick Start Guide for Recruitment Backend

## 📋 Prerequisites

You need ONE of these options:

### Option A: Docker (Recommended)
1. Install Docker Desktop from https://www.docker.com/products/docker-desktop/
2. Start Docker Desktop
3. Run: `docker-compose up`

### Option B: Local Rust + PostgreSQL
1. Install Rust: https://rustup.rs/
2. Install PostgreSQL: https://www.postgresql.org/download/windows/
3. Create database: `createdb recruitment`
4. Run: `cargo run`

## 🏃‍♂️ Quick Commands

### Using Docker
```bash
# Start all services
docker-compose up

# Start in background
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Using Local Setup
```bash
# Install dependencies (first time only)
cargo build

# Run the server
cargo run

# Run tests
cargo test

# Development with auto-reload
cargo install cargo-watch
cargo watch -x run
```

## 🗄️ Database Setup

### With Docker (Automatic)
Database is created automatically when you run `docker-compose up`

### Manual Setup
```sql
-- Create database
CREATE DATABASE recruitment;

-- Run the schema
psql -d recruitment -f database.sql
```

## 🌐 API Endpoints

Once running, the API will be available at: `http://localhost:3000`

### Health Check
```bash
curl http://localhost:3000/
```

### Authentication
```bash
# Register admin
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"Admin123!","role":"admin"}'

# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"Admin123!"}'
```

## 🔧 Configuration

Environment variables in `.env`:
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret for JWT tokens
- `RUST_LOG`: Logging level (debug, info, warn, error)

## 📊 Sample Data

The database includes sample data:
- 2 client companies
- 3 workers with skills
- 2 job postings

## 🐛 Troubleshooting

### Port Already in Use
```bash
# Kill process on port 3000 (Windows)
netstat -ano | findstr :3000
taskkill /PID <PID> /F
```

### Database Connection Issues
- Ensure PostgreSQL is running
- Check DATABASE_URL in .env
- Verify database exists

### Docker Issues
- Restart Docker Desktop
- Run `docker system prune` to clean up
- Check Docker logs: `docker-compose logs`

## 📚 API Documentation

Full API documentation available at: `http://localhost:3000/docs` (when running)

### Main Endpoints
- `GET /` - Health check
- `POST /api/auth/register` - Register user
- `POST /api/auth/login` - Login
- `GET /api/clients` - List clients
- `GET /api/workers` - List workers
- `GET /api/jobs` - List jobs
- `GET /api/meetings` - List meetings
- `GET /api/match/job/:id` - Find matches for job

## 🚨 Security Notes

- Change JWT_SECRET in production
- Use HTTPS in production
- Enable rate limiting
- Set up proper CORS origins

---

**Need help?** Check the full README.md for detailed documentation.
