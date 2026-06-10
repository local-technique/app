ALTER TABLE maintenances RENAME COLUMN code TO key;
ALTER TABLE incidents RENAME COLUMN code TO key;
ALTER TABLE projects RENAME COLUMN code TO key;
ALTER TABLE event_categories RENAME COLUMN code TO key;

ALTER TABLE incidents DROP CONSTRAINT IF EXISTS incidents_category_code_fkey;
ALTER TABLE maintenances DROP CONSTRAINT IF EXISTS maintenances_category_code_fkey;
ALTER TABLE projects DROP CONSTRAINT IF EXISTS projects_category_code_fkey;
ALTER TABLE event_category_i18n DROP CONSTRAINT IF EXISTS event_category_i18n_category_id_fkey;
ALTER TABLE event_category_i18n DROP CONSTRAINT IF EXISTS event_category_i18n_pkey;

ALTER TABLE event_categories ADD COLUMN IF NOT EXISTS uuid_id UUID;
UPDATE event_categories SET uuid_id = gen_random_uuid() WHERE uuid_id IS NULL;

ALTER TABLE incidents ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE incidents i SET category_uuid = c.uuid_id FROM event_categories c WHERE i.category_code = c.id;

ALTER TABLE maintenances ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE maintenances m SET category_uuid = c.uuid_id FROM event_categories c WHERE m.category_code = c.id;

ALTER TABLE projects ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE projects p SET category_uuid = c.uuid_id FROM event_categories c WHERE p.category_code = c.id;

ALTER TABLE event_category_i18n ADD COLUMN IF NOT EXISTS category_uuid UUID;
UPDATE event_category_i18n ci SET category_uuid = c.uuid_id FROM event_categories c WHERE ci.category_id = c.id;

ALTER TABLE incidents ALTER COLUMN category_uuid SET NOT NULL;
ALTER TABLE maintenances ALTER COLUMN category_uuid SET NOT NULL;
ALTER TABLE projects ALTER COLUMN category_uuid SET NOT NULL;
ALTER TABLE event_category_i18n ALTER COLUMN category_uuid SET NOT NULL;

ALTER TABLE incidents DROP COLUMN category_code;
ALTER TABLE maintenances DROP COLUMN category_code;
ALTER TABLE projects DROP COLUMN category_code;
ALTER TABLE event_category_i18n DROP COLUMN category_id;

ALTER TABLE incidents RENAME COLUMN category_uuid TO category_id;
ALTER TABLE maintenances RENAME COLUMN category_uuid TO category_id;
ALTER TABLE projects RENAME COLUMN category_uuid TO category_id;
ALTER TABLE event_category_i18n RENAME COLUMN category_uuid TO category_id;

ALTER TABLE event_categories DROP CONSTRAINT event_categories_pkey;
ALTER TABLE event_categories DROP COLUMN id;
ALTER TABLE event_categories RENAME COLUMN uuid_id TO id;
ALTER TABLE event_categories ADD PRIMARY KEY (id);
ALTER TABLE event_category_i18n ADD PRIMARY KEY (category_id, locale);

ALTER TABLE event_category_i18n ADD CONSTRAINT event_category_i18n_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE CASCADE;
ALTER TABLE incidents ADD CONSTRAINT incidents_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE RESTRICT;
ALTER TABLE maintenances ADD CONSTRAINT maintenances_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE RESTRICT;
ALTER TABLE projects ADD CONSTRAINT projects_category_id_fkey FOREIGN KEY (category_id) REFERENCES event_categories(id) ON DELETE RESTRICT;

CREATE SEQUENCE IF NOT EXISTS maintenance_key_seq;
CREATE SEQUENCE IF NOT EXISTS incident_key_seq;
CREATE SEQUENCE IF NOT EXISTS project_key_seq;

SELECT setval('maintenance_key_seq', greatest(9, coalesce((SELECT max((regexp_match(key, '^EVT-([1-9][0-9]*)$'))[1]::bigint) FROM maintenances WHERE key ~ '^EVT-[1-9][0-9]*$'), 9)), true);
SELECT setval('incident_key_seq', greatest(9, coalesce((SELECT max((regexp_match(key, '^INC-([1-9][0-9]*)$'))[1]::bigint) FROM incidents WHERE key ~ '^INC-[1-9][0-9]*$'), 9)), true);
SELECT setval('project_key_seq', greatest(9, coalesce((SELECT max((regexp_match(key, '^PRJ-([1-9][0-9]*)$'))[1]::bigint) FROM projects WHERE key ~ '^PRJ-[1-9][0-9]*$'), 9)), true);

CREATE INDEX IF NOT EXISTS incidents_category_idx ON incidents(category_id);
CREATE INDEX IF NOT EXISTS maintenances_category_idx ON maintenances(category_id);
DROP INDEX IF EXISTS projects_category_idx;
CREATE INDEX IF NOT EXISTS projects_category_idx ON projects(category_id);
