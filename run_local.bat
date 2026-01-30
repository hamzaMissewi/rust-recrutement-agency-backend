@echo off
echo Starting Recruitment Backend...

REM Check if Rust is installed
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Rust is not installed. Please install from https://rustup.rs/
    pause
    exit /b 1
)

REM Check if PostgreSQL is running
pg_isready -U postgres >nul 2>&1
if %errorlevel% neq 0 (
    echo PostgreSQL is not running. Please start PostgreSQL service.
    pause
    exit /b 1
)

REM Check if database exists
psql -U postgres -lqt | cut -d \| -f 1 | grep -qw recruitment
if %errorlevel% neq 0 (
    echo Database not found. Running setup...
    call setup_database.bat
)

echo Starting server...
cargo run
