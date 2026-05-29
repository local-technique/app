CREATE TABLE IF NOT EXISTS translation_keys (
  key_name TEXT PRIMARY KEY,
  description TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS translation_values (
  key_name TEXT NOT NULL REFERENCES translation_keys(key_name) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  value TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (key_name, locale)
);
