-- Enable UUID extension (provides gen_random_uuid() function)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Grant permissions (adjust as needed for your setup)
GRANT ALL PRIVILEGES ON DATABASE sentinel_auth TO postgres;
GRANT ALL ON ALL TABLES IN SCHEMA public TO postgres;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO postgres;
GRANT ALL ON ALL FUNCTIONS IN SCHEMA public TO postgres;

-- Set default permissions for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO postgres;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO postgres;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON FUNCTIONS TO postgres;

-- Create a read-only user for your application (optional)
-- CREATE USER sentinel_app WITH PASSWORD 'your_secure_password';
-- GRANT CONNECT ON DATABASE sentinel_auth TO sentinel_app;
-- GRANT USAGE ON SCHEMA public TO sentinel_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO sentinel_app;
-- GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO sentinel_app;
