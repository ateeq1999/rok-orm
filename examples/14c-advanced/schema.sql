-- Schema for 14c-advanced examples
-- Run: docker compose up -d && docker compose exec postgres psql -U rok -d rok_orm_14c -f /docker-entrypoint-initdb.d/schema.sql

-- Example users table
CREATE TABLE IF NOT EXISTS example_users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    age INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Example products table
CREATE TABLE IF NOT EXISTS example_products (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    category VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Posts table for full-text search
CREATE TABLE IF NOT EXISTS posts (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    body TEXT,
    author_id BIGINT REFERENCES example_users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Demo tables for dynamic creation (subqueries, window functions, etc.)
CREATE TABLE IF NOT EXISTS demo_users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    department VARCHAR(50),
    salary DECIMAL(10, 2)
);

CREATE TABLE IF NOT EXISTS demo_orders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    total DECIMAL(10, 2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS demo_departments (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    budget DECIMAL(12, 2)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_example_users_email ON example_users(email);
CREATE INDEX IF NOT EXISTS idx_example_products_category ON example_products(category);
CREATE INDEX IF NOT EXISTS idx_posts_author_id ON posts(author_id);
CREATE INDEX IF NOT EXISTS idx_posts_fts ON posts USING gin(to_tsvector('english', title || ' ' COALESCE(body, '')));
CREATE INDEX IF NOT EXISTS idx_demo_users_department ON demo_users(department);