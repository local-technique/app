CREATE TABLE IF NOT EXISTS project_timeline (
  id UUID PRIMARY KEY,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  at_utc TIMESTAMPTZ,
  sort_order INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (project_id, sort_order)
);

CREATE TABLE IF NOT EXISTS project_timeline_i18n (
  timeline_id UUID NOT NULL REFERENCES project_timeline(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (timeline_id, locale, field_key)
);

CREATE INDEX IF NOT EXISTS project_timeline_project_idx ON project_timeline(project_id, sort_order, at_utc);
CREATE INDEX IF NOT EXISTS project_timeline_i18n_lookup_idx ON project_timeline_i18n(timeline_id, locale, field_key);

CREATE TABLE IF NOT EXISTS maintenance_timeline (
  id UUID PRIMARY KEY,
  maintenance_id UUID NOT NULL REFERENCES maintenances(id) ON DELETE CASCADE,
  at_utc TIMESTAMPTZ,
  sort_order INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (maintenance_id, sort_order)
);

CREATE TABLE IF NOT EXISTS maintenance_timeline_i18n (
  timeline_id UUID NOT NULL REFERENCES maintenance_timeline(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (timeline_id, locale, field_key)
);

CREATE INDEX IF NOT EXISTS maintenance_timeline_maintenance_idx ON maintenance_timeline(maintenance_id, sort_order, at_utc);
CREATE INDEX IF NOT EXISTS maintenance_timeline_i18n_lookup_idx ON maintenance_timeline_i18n(timeline_id, locale, field_key);
