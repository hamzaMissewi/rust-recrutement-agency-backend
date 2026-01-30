-- Create database schema for recruitment agency

-- Clients table (recruitment companies)
CREATE TABLE IF NOT EXISTS clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Workers table (job candidates)
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
    job_type VARCHAR(50) DEFAULT 'full-time', -- full-time, part-time, contract, remote
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Applications table (workers applying to jobs)
CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES job_postings(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    status VARCHAR(50) DEFAULT 'pending', -- pending, reviewing, accepted, rejected
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
    status VARCHAR(50) DEFAULT 'scheduled', -- scheduled, completed, cancelled, rescheduled
    meeting_url VARCHAR(500), -- For virtual meetings
    location VARCHAR(255), -- For in-person meetings
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL, -- admin, client, worker
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    worker_id UUID REFERENCES workers(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_workers_skills ON workers USING GIN(skills);
CREATE INDEX IF NOT EXISTS idx_job_postings_requirements ON job_postings USING GIN(requirements);
CREATE INDEX IF NOT EXISTS idx_job_postings_client_id ON job_postings(client_id);
CREATE INDEX IF NOT EXISTS idx_job_postings_is_active ON job_postings(is_active);
CREATE INDEX IF NOT EXISTS idx_applications_job_id ON applications(job_id);
CREATE INDEX IF NOT EXISTS idx_applications_worker_id ON applications(worker_id);
CREATE INDEX IF NOT EXISTS idx_meetings_client_id ON meetings(client_id);
CREATE INDEX IF NOT EXISTS idx_meetings_worker_id ON meetings(worker_id);
CREATE INDEX IF NOT EXISTS idx_meetings_scheduled_at ON meetings(scheduled_at);

-- Insert sample data for testing
INSERT INTO clients (company_name, email, phone) VALUES 
('Tech Solutions Inc', 'contact@techsolutions.com', '+1-555-0101'),
('Global Recruitment', 'info@globalrecruit.com', '+1-555-0102')
ON CONFLICT (email) DO NOTHING;

INSERT INTO workers (name, email, phone, skills, experience_years) VALUES 
('John Doe', 'john.doe@email.com', '+1-555-0201', ARRAY['JavaScript', 'React', 'Node.js'], 5),
('Jane Smith', 'jane.smith@email.com', '+1-555-0202', ARRAY['Python', 'Django', 'PostgreSQL'], 3),
('Mike Johnson', 'mike.johnson@email.com', '+1-555-0203', ARRAY['Java', 'Spring', 'MongoDB'], 7)
ON CONFLICT (email) DO NOTHING;

INSERT INTO job_postings (client_id, title, description, requirements, salary_range, location) VALUES 
((SELECT id FROM clients WHERE email = 'contact@techsolutions.com'), 'Senior Frontend Developer', 'We are looking for an experienced frontend developer to join our team.', ARRAY['JavaScript', 'React', 'TypeScript'], '$80,000 - $120,000', 'Remote'),
((SELECT id FROM clients WHERE email = 'info@globalrecruit.com'), 'Backend Engineer', 'Seeking a skilled backend engineer to work on our core systems.', ARRAY['Python', 'Django', 'PostgreSQL'], '$70,000 - $100,000', 'New York')
ON CONFLICT DO NOTHING;
