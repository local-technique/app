# Database and i18n Brainstorming (Single Translation Table)

## Goals

- Keep auth persistence simple (`users`, merged `auth_sessions`).
- Keep incidents and maintenances separated (no business-table factorization).
- Use one single translation table where one row is one translation key and all locales live in one JSON object.
- Support locale priority fallback (`de -> en -> fr`) directly in SQL.
- Return API-ready payloads from SQL for end-user responses.

## Translation key convention

- Incident fields: `incident.{incident_id}.{field}`
- Incident timeline fields: `incident_timeline.{timeline_id}.{field}`
- Maintenance/event fields: `maintenance.{maintenance_id}.{field}`
- UI keys: `ui.{key_name}` (example: `ui.nav.events`)

Field names are free-form strings (`title`, `short_description`, `long_description`, `location`, `warning`, `details`, ...).

## Proposed schema (plain SQL)

```sql
CREATE TABLE users (
  id UUID PRIMARY KEY,
  email CITEXT NOT NULL UNIQUE,
  provider TEXT NOT NULL CHECK (provider IN ('google', 'facebook')),
  roles TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
CREATE TABLE auth_sessions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  refresh_token_hash TEXT NOT NULL,
  previous_refresh_token_hashes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
  expires_at TIMESTAMPTZ NOT NULL,
  revoked_at TIMESTAMPTZ,
  compromised_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
CREATE TABLE locales (
  code TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
CREATE TABLE incidents (
  id UUID PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  category_code TEXT NOT NULL,
  start_utc TIMESTAMPTZ NOT NULL,
  end_utc TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
CREATE TABLE incident_timeline (
  id UUID PRIMARY KEY,
  incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
  at_utc TIMESTAMPTZ NOT NULL,
  sort_order INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
CREATE TABLE maintenances (
  id UUID PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,
  category_code TEXT NOT NULL,
  start_utc TIMESTAMPTZ NOT NULL,
  end_utc TIMESTAMPTZ,
  notified_at_utc TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
CREATE TABLE translations (
  key_name TEXT PRIMARY KEY,
  translations JSONB NOT NULL DEFAULT '{}'::JSONB,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT translations_is_object_chk CHECK (jsonb_typeof(translations) = 'object')
);
```

## Recommended indexes

```sql
CREATE INDEX auth_sessions_user_id_idx ON auth_sessions(user_id);
CREATE INDEX auth_sessions_expires_at_idx ON auth_sessions(expires_at);
CREATE UNIQUE INDEX auth_sessions_refresh_token_hash_uq ON auth_sessions(refresh_token_hash);

CREATE INDEX incident_timeline_incident_idx ON incident_timeline(incident_id, sort_order, at_utc);

CREATE INDEX translations_key_name_pattern_idx ON translations(key_name text_pattern_ops);
CREATE INDEX translations_jsonb_gin_idx ON translations USING GIN (translations jsonb_path_ops);
```

## Locale-fallback conventions used by queries

- Locale priority is passed as one array parameter, for example: `ARRAY['de','en','fr']::TEXT[]`.
- For each field, SQL picks the first non-empty translation value in that priority order.

---

## Incidents queries

### Select all incidents (list/minimal, i18n-sensitive)

```sql
-- $1 :: TEXT[] locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($1::TEXT[]) WITH ORDINALITY AS t(locale, ord)
)
SELECT
  i.id,
  i.code,
  i.category_code,
  i.start_utc,
  i.end_utc,
  title.value AS title,
  short_desc.value AS short_description,
  location.value AS location
FROM incidents i
LEFT JOIN LATERAL (
  SELECT t.translations ->> lp.locale AS value
  FROM locale_priority lp
  JOIN translations t ON t.key_name = format('incident.%s.title', i.id)
  WHERE coalesce(t.translations ->> lp.locale, '') <> ''
  ORDER BY lp.ord
  LIMIT 1
) title ON TRUE
LEFT JOIN LATERAL (
  SELECT t.translations ->> lp.locale AS value
  FROM locale_priority lp
  JOIN translations t ON t.key_name = format('incident.%s.short_description', i.id)
  WHERE coalesce(t.translations ->> lp.locale, '') <> ''
  ORDER BY lp.ord
  LIMIT 1
) short_desc ON TRUE
LEFT JOIN LATERAL (
  SELECT t.translations ->> lp.locale AS value
  FROM locale_priority lp
  JOIN translations t ON t.key_name = format('incident.%s.location', i.id)
  WHERE coalesce(t.translations ->> lp.locale, '') <> ''
  ORDER BY lp.ord
  LIMIT 1
) location ON TRUE
ORDER BY i.start_utc DESC;
```

### Select one incident detail (all fields, i18n-sensitive, API-ready JSON)

```sql
-- $1 :: UUID   incident id
-- $2 :: TEXT[] locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($2::TEXT[]) WITH ORDINALITY AS t(locale, ord)
),
timeline_json AS (
  SELECT
    t.incident_id,
    jsonb_agg(
      jsonb_build_object(
        'id', t.id,
        'atUtc', t.at_utc,
        'title', (
          SELECT tr.translations ->> lp.locale
          FROM locale_priority lp
          JOIN translations tr ON tr.key_name = format('incident_timeline.%s.title', t.id)
          WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
          ORDER BY lp.ord
          LIMIT 1
        ),
        'details', (
          SELECT tr.translations ->> lp.locale
          FROM locale_priority lp
          JOIN translations tr ON tr.key_name = format('incident_timeline.%s.details', t.id)
          WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
          ORDER BY lp.ord
          LIMIT 1
        )
      )
      ORDER BY t.sort_order, t.at_utc
    ) AS timeline
  FROM incident_timeline t
  WHERE t.incident_id = $1
  GROUP BY t.incident_id
)
SELECT jsonb_build_object(
  'id', i.id,
  'code', i.code,
  'categoryCode', i.category_code,
  'startUtc', i.start_utc,
  'endUtc', i.end_utc,
  'title', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('incident.%s.title', i.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'shortDescription', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('incident.%s.short_description', i.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'longDescription', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('incident.%s.long_description', i.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'location', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('incident.%s.location', i.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'timeline', coalesce(tj.timeline, '[]'::JSONB)
) AS payload
FROM incidents i
LEFT JOIN timeline_json tj ON tj.incident_id = i.id
WHERE i.id = $1;
```

### Select all translations for one incident (role-specific operation)

```sql
-- $1 :: UUID incident id
WITH incident_keys AS (
  SELECT format('incident.%s.%%', $1::TEXT) AS pattern
),
timeline_keys AS (
  SELECT format('incident_timeline.%s.%%', t.id::TEXT) AS pattern
  FROM incident_timeline t
  WHERE t.incident_id = $1
),
all_patterns AS (
  SELECT pattern FROM incident_keys
  UNION ALL
  SELECT pattern FROM timeline_keys
)
SELECT
  tr.key_name,
  tr.translations,
  tr.updated_at
FROM translations tr
WHERE EXISTS (
  SELECT 1
  FROM all_patterns p
  WHERE tr.key_name LIKE p.pattern
)
ORDER BY tr.key_name;
```

### Update all translations for one incident (replace set, batched, single statement)

```sql
-- $1 :: UUID   incident id
-- $2 :: JSONB  payload array of objects
-- Example element:
-- {
--   "key_name": "incident.<id>.title",
--   "translations": {"en":"Heating outage","fr":"Panne chauffage"}
-- }
WITH incoming_raw AS (
  SELECT
    x.key_name::TEXT AS key_name,
    x.translations::JSONB AS translations
  FROM jsonb_to_recordset($2) AS x(key_name TEXT, translations JSONB)
),
scope_keys AS (
  SELECT format('incident.%s.%%', $1::TEXT) AS pattern
  UNION ALL
  SELECT format('incident_timeline.%s.%%', t.id::TEXT)
  FROM incident_timeline t
  WHERE t.incident_id = $1
),
incoming AS (
  SELECT ir.key_name, ir.translations
  FROM incoming_raw ir
  WHERE EXISTS (
    SELECT 1
    FROM scope_keys sk
    WHERE ir.key_name LIKE sk.pattern
  )
),
deleted AS (
  DELETE FROM translations tr
  WHERE EXISTS (
      SELECT 1
      FROM scope_keys sk
      WHERE tr.key_name LIKE sk.pattern
    )
    AND NOT EXISTS (
      SELECT 1
      FROM incoming i
      WHERE i.key_name = tr.key_name
    )
  RETURNING 1
)
INSERT INTO translations (key_name, translations, updated_at)
SELECT i.key_name, i.translations, now()
FROM incoming i
ON CONFLICT (key_name)
DO UPDATE SET
  translations = EXCLUDED.translations,
  updated_at = now();
```

---

## Maintenances (events) queries

### Select all maintenances/events (list/minimal, i18n-sensitive)

```sql
-- $1 :: TEXT[] locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($1::TEXT[]) WITH ORDINALITY AS t(locale, ord)
)
SELECT
  m.id,
  m.code,
  m.category_code,
  m.start_utc,
  m.end_utc,
  m.notified_at_utc,
  title.value AS title,
  short_desc.value AS short_description,
  warning.value AS warning,
  location.value AS location
FROM maintenances m
LEFT JOIN LATERAL (
  SELECT t.translations ->> lp.locale AS value
  FROM locale_priority lp
  JOIN translations t ON t.key_name = format('maintenance.%s.title', m.id)
  WHERE coalesce(t.translations ->> lp.locale, '') <> ''
  ORDER BY lp.ord
  LIMIT 1
) title ON TRUE
LEFT JOIN LATERAL (
  SELECT t.translations ->> lp.locale AS value
  FROM locale_priority lp
  JOIN translations t ON t.key_name = format('maintenance.%s.short_description', m.id)
  WHERE coalesce(t.translations ->> lp.locale, '') <> ''
  ORDER BY lp.ord
  LIMIT 1
) short_desc ON TRUE
LEFT JOIN LATERAL (
  SELECT t.translations ->> lp.locale AS value
  FROM locale_priority lp
  JOIN translations t ON t.key_name = format('maintenance.%s.warning', m.id)
  WHERE coalesce(t.translations ->> lp.locale, '') <> ''
  ORDER BY lp.ord
  LIMIT 1
) warning ON TRUE
LEFT JOIN LATERAL (
  SELECT t.translations ->> lp.locale AS value
  FROM locale_priority lp
  JOIN translations t ON t.key_name = format('maintenance.%s.location', m.id)
  WHERE coalesce(t.translations ->> lp.locale, '') <> ''
  ORDER BY lp.ord
  LIMIT 1
) location ON TRUE
ORDER BY m.start_utc DESC;
```

### Select one maintenance/event detail (all fields, i18n-sensitive, API-ready JSON)

```sql
-- $1 :: UUID   maintenance id
-- $2 :: TEXT[] locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($2::TEXT[]) WITH ORDINALITY AS t(locale, ord)
)
SELECT jsonb_build_object(
  'id', m.id,
  'code', m.code,
  'categoryCode', m.category_code,
  'startUtc', m.start_utc,
  'endUtc', m.end_utc,
  'notifiedAtUtc', m.notified_at_utc,
  'title', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('maintenance.%s.title', m.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'shortDescription', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('maintenance.%s.short_description', m.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'longDescription', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('maintenance.%s.long_description', m.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'warning', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('maintenance.%s.warning', m.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  ),
  'location', (
    SELECT tr.translations ->> lp.locale
    FROM locale_priority lp
    JOIN translations tr ON tr.key_name = format('maintenance.%s.location', m.id)
    WHERE coalesce(tr.translations ->> lp.locale, '') <> ''
    ORDER BY lp.ord
    LIMIT 1
  )
) AS payload
FROM maintenances m
WHERE m.id = $1;
```

### Select all translations for one maintenance/event (role-specific operation)

```sql
-- $1 :: UUID maintenance id
SELECT
  tr.key_name,
  tr.translations,
  tr.updated_at
FROM translations tr
WHERE tr.key_name LIKE format('maintenance.%s.%%', $1::TEXT)
ORDER BY tr.key_name;
```

### Update all translations for one maintenance/event (replace set, batched, single statement)

```sql
-- $1 :: UUID   maintenance id
-- $2 :: JSONB  payload array of objects
-- Example element:
-- {
--   "key_name": "maintenance.<id>.title",
--   "translations": {"en":"Heating maintenance","fr":"Maintenance chauffage"}
-- }
WITH incoming_raw AS (
  SELECT
    x.key_name::TEXT AS key_name,
    x.translations::JSONB AS translations
  FROM jsonb_to_recordset($2) AS x(key_name TEXT, translations JSONB)
),
incoming AS (
  SELECT ir.key_name, ir.translations
  FROM incoming_raw ir
  WHERE ir.key_name LIKE format('maintenance.%s.%%', $1::TEXT)
),
deleted AS (
  DELETE FROM translations tr
  WHERE tr.key_name LIKE format('maintenance.%s.%%', $1::TEXT)
    AND NOT EXISTS (
      SELECT 1
      FROM incoming i
      WHERE i.key_name = tr.key_name
    )
  RETURNING 1
)
INSERT INTO translations (key_name, translations, updated_at)
SELECT i.key_name, i.translations, now()
FROM incoming i
ON CONFLICT (key_name)
DO UPDATE SET
  translations = EXCLUDED.translations,
  updated_at = now();
```

---

## Generic key translation queries

### Select all existing keys and translations with missing values obvious

```sql
SELECT
  tr.key_name,
  l.code AS locale,
  tr.translations ->> l.code AS value,
  (coalesce(tr.translations ->> l.code, '') = '') AS is_missing
FROM translations tr
CROSS JOIN locales l
ORDER BY tr.key_name, l.code;
```

### Update all given translations for keys and locale (single batched update)

```sql
-- $1 :: JSONB payload array
-- Example element: {"key_name":"ui.nav.events","locale":"de","value":"Ereignisse"}
WITH incoming AS (
  SELECT
    x.key_name::TEXT AS key_name,
    x.locale::TEXT AS locale,
    x.value::TEXT AS value
  FROM jsonb_to_recordset($1) AS x(key_name TEXT, locale TEXT, value TEXT)
),
patch_per_key AS (
  SELECT
    key_name,
    jsonb_object_agg(locale, to_jsonb(value)) AS patch
  FROM incoming
  GROUP BY key_name
)
INSERT INTO translations (key_name, translations, updated_at)
SELECT p.key_name, p.patch, now()
FROM patch_per_key p
ON CONFLICT (key_name)
DO UPDATE
SET
  translations = translations.translations || EXCLUDED.translations,
  updated_at = now();
```

## Why this design serves the requested use cases

- Single translation table: one row per key, all locales in one JSON object.
- Fallback chain in SQL: locale-priority array drives per-field selection in a single query.
- Admin operations: easy full-key reads and batched upserts/replacements.
- Missing translations: explicit in `CROSS JOIN locales` query with `is_missing`.
- Field evolution: add new fields by adding new keys, no table migration needed.
