CREATE TABLE IF NOT EXISTS incidents (
  id UUID PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  category_code TEXT NOT NULL,
  start_utc TIMESTAMPTZ NOT NULL,
  end_utc TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS incident_i18n (
  incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (incident_id, locale, field_key)
);

CREATE TABLE IF NOT EXISTS incident_timeline (
  id UUID PRIMARY KEY,
  incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
  at_utc TIMESTAMPTZ NOT NULL,
  sort_order INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS incident_timeline_i18n (
  timeline_id UUID NOT NULL REFERENCES incident_timeline(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (timeline_id, locale, field_key)
);

CREATE INDEX IF NOT EXISTS incident_i18n_lookup_idx ON incident_i18n(incident_id, field_key, locale);
CREATE INDEX IF NOT EXISTS incident_timeline_incident_idx ON incident_timeline(incident_id, sort_order, at_utc);
CREATE INDEX IF NOT EXISTS incident_timeline_i18n_lookup_idx ON incident_timeline_i18n(timeline_id, field_key, locale);
