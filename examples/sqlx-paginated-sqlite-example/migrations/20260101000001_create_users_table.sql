-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    confirmed BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Create index on email for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Create index on confirmed for filtering
CREATE INDEX IF NOT EXISTS idx_users_confirmed ON users(confirmed);

-- Create index on created_at for sorting
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

