-- Add password field to users table
ALTER TABLE users ADD COLUMN password_hash VARCHAR NOT NULL DEFAULT '';
