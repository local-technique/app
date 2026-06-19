ALTER TABLE incidents ADD COLUMN status_type TEXT NOT NULL DEFAULT 'ongoing' CHECK (status_type IN ('waiting', 'ongoing'));

ALTER TABLE maintenances ADD COLUMN status_type TEXT NOT NULL DEFAULT 'ongoing' CHECK (status_type IN ('waiting', 'ongoing'));

CREATE INDEX IF NOT EXISTS incidents_status_type_idx ON incidents(status_type);
CREATE INDEX IF NOT EXISTS maintenances_status_type_idx ON maintenances(status_type);
