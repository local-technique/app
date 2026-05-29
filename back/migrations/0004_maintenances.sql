CREATE TABLE IF NOT EXISTS maintenances (
  id UUID PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  category_code TEXT NOT NULL,
  start_utc TIMESTAMPTZ NOT NULL,
  end_utc TIMESTAMPTZ,
  notified_at_utc TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS maintenance_i18n (
  maintenance_id UUID NOT NULL REFERENCES maintenances(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (maintenance_id, locale, field_key)
);

CREATE INDEX IF NOT EXISTS maintenance_i18n_lookup_idx ON maintenance_i18n(maintenance_id, field_key, locale);
