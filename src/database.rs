use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub async fn create_connection_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/recruitment".to_string());

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create tables if they don't exist
    sqlx::query(
        r#"
        -- Clients table
        CREATE TABLE IF NOT EXISTS clients (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            company_name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            phone VARCHAR(50),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Workers table
        CREATE TABLE IF NOT EXISTS workers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            phone VARCHAR(50),
            skills TEXT[] DEFAULT '{}',
            experience_years INTEGER DEFAULT 0,
            resume_url VARCHAR(500),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Job postings table
        CREATE TABLE IF NOT EXISTS job_postings (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
            title VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            requirements TEXT[] DEFAULT '{}',
            salary_range VARCHAR(100),
            location VARCHAR(255) NOT NULL,
            job_type VARCHAR(50) DEFAULT 'full-time',
            is_active BOOLEAN DEFAULT true,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Applications table
        CREATE TABLE IF NOT EXISTS applications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            job_id UUID NOT NULL REFERENCES job_postings(id) ON DELETE CASCADE,
            worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
            status VARCHAR(50) DEFAULT 'pending',
            cover_letter TEXT,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            UNIQUE(job_id, worker_id)
        );

        -- Meetings table
        CREATE TABLE IF NOT EXISTS meetings (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
            worker_id UUID REFERENCES workers(id) ON DELETE SET NULL,
            job_id UUID REFERENCES job_postings(id) ON DELETE SET NULL,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            scheduled_at TIMESTAMP WITH TIME ZONE NOT NULL,
            duration_minutes INTEGER DEFAULT 60,
            status VARCHAR(50) DEFAULT 'scheduled',
            meeting_url VARCHAR(500),
            location VARCHAR(255),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Users table for authentication
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            role VARCHAR(50) NOT NULL,
            client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
            worker_id UUID REFERENCES workers(id) ON DELETE CASCADE,
            is_active BOOLEAN DEFAULT true,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_workers_skills ON workers USING GIN(skills);
        CREATE INDEX IF NOT EXISTS idx_job_postings_requirements ON job_postings USING GIN(requirements);
        CREATE INDEX IF NOT EXISTS idx_job_postings_client_id ON job_postings(client_id);
        CREATE INDEX IF NOT EXISTS idx_job_postings_is_active ON job_postings(is_active);
        CREATE INDEX IF NOT EXISTS idx_applications_job_id ON applications(job_id);
        CREATE INDEX IF NOT EXISTS idx_applications_worker_id ON applications(worker_id);
        CREATE INDEX IF NOT EXISTS idx_meetings_client_id ON meetings(client_id);
        CREATE INDEX IF NOT EXISTS idx_meetings_worker_id ON meetings(worker_id);
        CREATE INDEX IF NOT EXISTS idx_meetings_scheduled_at ON meetings(scheduled_at);
        "#
    )
    .execute(pool)
    .await?;

    // Insert sample data if tables are empty
    let client_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
        .fetch_one(pool)
        .await?;

    if client_count == 0 {
        sqlx::query(
            r#"
            INSERT INTO clients (company_name, email, phone) VALUES 
            ('Tech Solutions Inc', 'contact@techsolutions.com', '+1-555-0101'),
            ('Global Recruitment', 'info@globalrecruit.com', '+1-555-0102')
            "#
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO workers (name, email, phone, skills, experience_years) VALUES 
            ('John Doe', 'john.doe@email.com', '+1-555-0201', ARRAY['JavaScript', 'React', 'Node.js'], 5),
            ('Jane Smith', 'jane.smith@email.com', '+1-555-0202', ARRAY['Python', 'Django', 'PostgreSQL'], 3),
            ('Mike Johnson', 'mike.johnson@email.com', '+1-555-0203', ARRAY['Java', 'Spring', 'MongoDB'], 7)
            "#
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO job_postings (client_id, title, description, requirements, salary_range, location) 
            SELECT c.id, 'Senior Frontend Developer', 'We are looking for an experienced frontend developer to join our team.', 
                   ARRAY['JavaScript', 'React', 'TypeScript'], '$80,000 - $120,000', 'Remote'
            FROM clients c WHERE c.email = 'contact@techsolutions.com'
            LIMIT 1;
            
            INSERT INTO job_postings (client_id, title, description, requirements, salary_range, location) 
            SELECT c.id, 'Backend Engineer', 'Seeking a skilled backend engineer to work on our core systems.', 
                   ARRAY['Python', 'Django', 'PostgreSQL'], '$70,000 - $100,000', 'New York'
            FROM clients c WHERE c.email = 'info@globalrecruit.com'
            LIMIT 1;
            "#
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
