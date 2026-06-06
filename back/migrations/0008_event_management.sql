CREATE TABLE IF NOT EXISTS event_categories (
  id TEXT PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  icon TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS event_category_i18n (
  category_id TEXT NOT NULL REFERENCES event_categories(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  label TEXT NOT NULL,
  PRIMARY KEY (category_id, locale)
);

INSERT INTO event_categories (id, code, icon)
SELECT DISTINCT category_code, category_code,
  CASE category_code
    WHEN 'HEA' THEN 'flame'
    WHEN 'ELV' THEN 'arrow-up-down'
    WHEN 'PLB' THEN 'droplets'
    WHEN 'ELC' THEN 'zap'
    WHEN 'GAR' THEN 'warehouse'
    WHEN 'PMG' THEN 'building-2'
    ELSE 'tag'
  END
FROM (
  SELECT category_code FROM incidents
  UNION
  SELECT category_code FROM maintenances
) c
ON CONFLICT (id) DO NOTHING;

INSERT INTO event_category_i18n (category_id, locale, label)
SELECT c.id, l.code,
  CASE c.id
    WHEN 'HEA' THEN CASE l.code WHEN 'fr' THEN 'Chauffage' ELSE 'Heating' END
    WHEN 'ELV' THEN CASE l.code WHEN 'fr' THEN 'Ascenseur' ELSE 'Elevator' END
    WHEN 'PLB' THEN CASE l.code WHEN 'fr' THEN 'Plomberie' ELSE 'Plumbing' END
    WHEN 'ELC' THEN CASE l.code WHEN 'fr' THEN 'Electricite' ELSE 'Electrical' END
    WHEN 'GAR' THEN CASE l.code WHEN 'fr' THEN 'Garage' ELSE 'Garage' END
    WHEN 'PMG' THEN CASE l.code WHEN 'fr' THEN 'Syndic' ELSE 'Property management' END
    ELSE c.code
  END
FROM event_categories c
CROSS JOIN locales l
WHERE l.is_enabled = TRUE
ON CONFLICT (category_id, locale) DO NOTHING;

ALTER TABLE incidents ADD COLUMN IF NOT EXISTS last_modified_at TIMESTAMPTZ;
ALTER TABLE incidents ADD COLUMN IF NOT EXISTS last_modified_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE maintenances ADD COLUMN IF NOT EXISTS last_modified_at TIMESTAMPTZ;
ALTER TABLE maintenances ADD COLUMN IF NOT EXISTS last_modified_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL;

UPDATE incidents SET last_modified_at = updated_at WHERE last_modified_at IS NULL;
UPDATE maintenances SET last_modified_at = updated_at WHERE last_modified_at IS NULL;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'incidents_category_code_fkey'
  ) THEN
    ALTER TABLE incidents
      ADD CONSTRAINT incidents_category_code_fkey
      FOREIGN KEY (category_code) REFERENCES event_categories(id) ON DELETE RESTRICT;
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'maintenances_category_code_fkey'
  ) THEN
    ALTER TABLE maintenances
      ADD CONSTRAINT maintenances_category_code_fkey
      FOREIGN KEY (category_code) REFERENCES event_categories(id) ON DELETE RESTRICT;
  END IF;
END $$;
