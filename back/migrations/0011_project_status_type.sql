DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_name = 'projects' AND column_name = 'status'
  ) AND NOT EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_name = 'projects' AND column_name = 'status_type'
  ) THEN
    ALTER TABLE projects RENAME COLUMN status TO status_type;
  END IF;
END $$;

DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM pg_indexes
    WHERE indexname = 'projects_status_idx'
  ) AND NOT EXISTS (
    SELECT 1
    FROM pg_indexes
    WHERE indexname = 'projects_status_type_idx'
  ) THEN
    ALTER INDEX projects_status_idx RENAME TO projects_status_type_idx;
  END IF;
END $$;
