# 🚀 Complete Setup Guide - Run Backend Without Docker

## 📋 Step-by-Step Instructions

### **Step 1: Install Rust**
```powershell
# Run as Administrator in PowerShell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
.\install_rust.ps1
```

**OR Manual Install:**
1. Go to https://rustup.rs/
2. Download `rustup-init.exe`
3. Run it with default options
4. Restart PowerShell

### **Step 2: Install PostgreSQL**
```powershell
# Run as Administrator in PowerShell
.\install_postgresql.ps1
```

**OR Manual Install:**
1. Go to https://www.postgresql.org/download/windows/
2. Download and install PostgreSQL
3. Set password: `password`
4. Make sure service is running

### **Step 3: Setup Database**
```powershell
# Create database
createdb -U postgres recruitment

# Run schema
psql -U postgres -d recruitment -f database.sql
```

### **Step 4: Run the Backend**
```powershell
cargo run
```

## 🔧 Quick Commands

### **Check Installation**
```powershell
# Check Rust
cargo --version

# Check PostgreSQL
psql --version

# Check if PostgreSQL is running
pg_isready -U postgres
```

### **Database Operations**
```powershell
# Create database
createdb -U postgres recruitment

# Connect to database
psql -U postgres -d recruitment

# Run schema file
psql -U postgres -d recruitment -f database.sql

# Drop database (if needed)
dropdb -U postgres recruitment
```

### **Backend Operations**
```powershell
# Build project
cargo build

# Run project
cargo run

# Run tests
cargo test

# Development with auto-reload
cargo install cargo-watch
cargo watch -x run
```

## 🌐 Test the API

Once running, test these endpoints:

```powershell
# Health check
curl http://localhost:3000/

# Register admin
curl -X POST http://localhost:3000/api/auth/register `
  -H "Content-Type: application/json" `
  -d '{"email":"admin@example.com","password":"Admin123!","role":"admin"}'

# Login
curl -X POST http://localhost:3000/api/auth/login `
  -H "Content-Type: application/json" `
  -d '{"email":"admin@example.com","password":"Admin123!"}'
```

## 🐛 Troubleshooting

### **Rust Issues**
```powershell
# If cargo not found, restart PowerShell
# If still not found, add to PATH manually:
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
```

### **PostgreSQL Issues**
```powershell
# Start PostgreSQL service
net start postgresql-x64-15

# Check if running
pg_isready -U postgres

# Reset password (if needed)
psql -U postgres -c "ALTER USER postgres PASSWORD 'password';"
```

### **Port Issues**
```powershell
# Check what's using port 3000
netstat -ano | findstr :3000

# Kill process
taskkill /PID <PID> /F
```

### **Build Issues**
```powershell
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update
```

## 📁 Project Structure
```
backend/
├── src/
│   ├── main.rs           # Entry point
│   ├── handlers/        # API handlers
│   ├── models.rs        # Data models
│   ├── auth.rs          # Authentication
│   ├── database.rs      # Database setup
│   ├── error.rs         # Error handling
│   └── utils.rs         # Utilities
├── Cargo.toml           # Rust dependencies
├── database.sql        # Database schema
├── .env                 # Environment variables
└── README.md           # Documentation
```

## 🎯 Success Indicators

✅ **Rust Installed**: `cargo --version` shows version  
✅ **PostgreSQL Running**: `pg_isready` returns success  
✅ **Database Created**: `recruitment` database exists  
✅ **Schema Loaded**: Tables created with sample data  
✅ **Backend Running**: `cargo run` starts server on port 3000  

## 🚀 Final Test

If everything is working, you should see:
```
Server running on http://localhost:3000
Database connected successfully
```

And the health check should return:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-30T..."
}
```

---

**Need help?** Check the logs or run `cargo run --verbose` for detailed output.
