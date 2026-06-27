-- Migrate any existing facebook users to microsoft
UPDATE users SET provider = 'microsoft' WHERE provider = 'facebook';

-- Drop ALL check constraints on users that reference provider (handles system-generated names)
DO $$
DECLARE
    cons RECORD;
BEGIN
    FOR cons IN (
        SELECT conname
        FROM pg_constraint
        WHERE conrelid = 'users'::regclass
          AND contype = 'c'
          AND pg_get_constraintdef(oid) LIKE '%provider%'
    ) LOOP
        EXECUTE format('ALTER TABLE users DROP CONSTRAINT %I', cons.conname);
    END LOOP;
END $$;

ALTER TABLE users ADD CONSTRAINT users_provider_check CHECK (provider IN ('google', 'microsoft'));
