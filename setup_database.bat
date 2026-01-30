@echo off
echo Setting up recruitment database...

REM Create database
createdb -U postgres recruitment

REM Run schema setup
psql -U postgres -d recruitment -f database.sql

echo Database setup complete!
echo You can now run: cargo run
