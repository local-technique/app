ALTER TABLE incident_timeline ADD COLUMN last_modified_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE maintenance_timeline ADD COLUMN last_modified_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE project_timeline ADD COLUMN last_modified_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
