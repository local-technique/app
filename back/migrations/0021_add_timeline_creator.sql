ALTER TABLE incident_timeline ADD COLUMN created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE maintenance_timeline ADD COLUMN created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE project_timeline ADD COLUMN created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;

UPDATE incident_timeline SET created_by_user_id = last_modified_by_user_id WHERE created_by_user_id IS NULL;
UPDATE maintenance_timeline SET created_by_user_id = last_modified_by_user_id WHERE created_by_user_id IS NULL;
UPDATE project_timeline SET created_by_user_id = last_modified_by_user_id WHERE created_by_user_id IS NULL;
