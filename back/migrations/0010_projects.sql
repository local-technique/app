CREATE TABLE IF NOT EXISTS projects (
  id UUID PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  category_code TEXT NOT NULL REFERENCES event_categories(id) ON DELETE RESTRICT,
  start_utc TIMESTAMPTZ,
  end_utc TIMESTAMPTZ,
  status TEXT NOT NULL CHECK (status IN ('waiting', 'ongoing')),
  last_modified_at TIMESTAMPTZ,
  last_modified_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS project_i18n (
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (project_id, locale, field_key)
);

CREATE INDEX IF NOT EXISTS projects_category_idx ON projects(category_code);
CREATE INDEX IF NOT EXISTS projects_status_idx ON projects(status);
CREATE INDEX IF NOT EXISTS project_i18n_lookup_idx ON project_i18n(project_id, field_key, locale);
