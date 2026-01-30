@echo off
echo ========================================
echo   Recruitment Backend Startup Script
echo ========================================
echo.

REM Check if Rust is installed
echo [1/4] Checking Rust installation...
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Rust not found. Installing Rust...
    echo.
    echo Please run this as Administrator:
    echo powershell -ExecutionPolicy Bypass -File install_rust.ps1
    echo.
    pause
    exit /b 1
)
echo ✅ Rust found: 
cargo --version
echo.

REM Check if PostgreSQL is installed
echo [2/4] Checking PostgreSQL installation...
psql --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ PostgreSQL not found.
    echo.
    echo Please install PostgreSQL from: https://www.postgresql.org/download/windows/
    echo Or run: powershell -ExecutionPolicy Bypass -File install_postgresql.ps1
    echo.
    pause
    exit /b 1
)
echo ✅ PostgreSQL found:
psql --version
echo.

REM Check if PostgreSQL is running
echo [3/4] Checking PostgreSQL service...
pg_isready -U postgres >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ PostgreSQL is not running.
    echo.
    echo Please start PostgreSQL service or run:
    echo net start postgresql-x64-15
    echo.
    pause
    exit /b 1
)
echo ✅ PostgreSQL is running
echo.

REM Check if database exists
echo [4/4] Setting up database...
psql -U postgres -lqt | cut -d \| -f 1 | findstr /C:"recruitment" >nul
if %errorlevel% neq 0 (
    echo 📊 Creating database...
    createdb -U postgres recruitment
    echo 📊 Loading schema...
    psql -U postgres -d recruitment -f database.sql
    echo ✅ Database setup complete
) else (
    echo ✅ Database already exists
)
echo.

echo ========================================
echo   Starting Recruitment Backend...
echo ========================================
echo.
echo 🚀 Server will be available at: http://localhost:3000
echo 📋 API Documentation: http://localhost:3000/
echo.
echo Press Ctrl+C to stop the server
echo.

cargo run
