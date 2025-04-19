-- Remove password field from users table
ALTER TABLE users DROP COLUMN password_hash;
