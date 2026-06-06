ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS users_roles_gin_idx ON users USING GIN (roles);
CREATE INDEX IF NOT EXISTS users_last_login_at_idx ON users(last_login_at);
