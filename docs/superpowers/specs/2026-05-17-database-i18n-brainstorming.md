# Database and i18n Brainstorming (PostgreSQL + SQLx migrations)

## Goals

- Keep auth persistence simple (`users`, merged `auth_sessions` with refresh rotation data).
- Keep incidents and maintenances separated (no shared business table).
- Make i18n evolvable without schema migrations for every new translatable field.
- Allow locale fallback chains like `de -> en -> fr` in a single SQL query.
- Return API-ready payloads directly from SQL for user-facing reads.

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
CREATE TABLE incident_i18n (
  incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (incident_id, locale, field_key)
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
CREATE TABLE incident_timeline_i18n (
  timeline_id UUID NOT NULL REFERENCES incident_timeline(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (timeline_id, locale, field_key)
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
CREATE TABLE maintenance_i18n (
  maintenance_id UUID NOT NULL REFERENCES maintenances(id) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  field_key TEXT NOT NULL,
  field_value TEXT NOT NULL,
  PRIMARY KEY (maintenance_id, locale, field_key)
);
```

```sql
CREATE TABLE translation_keys (
  key_name TEXT PRIMARY KEY,
  description TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```sql
CREATE TABLE translation_values (
  key_name TEXT NOT NULL REFERENCES translation_keys(key_name) ON DELETE CASCADE,
  locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT,
  value TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (key_name, locale)
);
```

## Recommended indexes

```sql
CREATE INDEX auth_sessions_user_id_idx ON auth_sessions(user_id);
CREATE INDEX auth_sessions_expires_at_idx ON auth_sessions(expires_at);
CREATE UNIQUE INDEX auth_sessions_refresh_token_hash_uq ON auth_sessions(refresh_token_hash);

CREATE INDEX incident_i18n_lookup_idx ON incident_i18n(incident_id, field_key, locale);
CREATE INDEX incident_timeline_incident_idx ON incident_timeline(incident_id, sort_order, at_utc);
CREATE INDEX incident_timeline_i18n_lookup_idx ON incident_timeline_i18n(timeline_id, field_key, locale);

CREATE INDEX maintenance_i18n_lookup_idx ON maintenance_i18n(maintenance_id, field_key, locale);
```

## Locale-fallback conventions used by queries

- `$1`, `$2`, etc are SQL bind parameters.
- Locale priority comes as one array parameter: `ARRAY['de','en','fr']::TEXT[]`.
- For each field, the first available translation in the priority list wins.

---

## Incidents queries

### Select all incidents (list/minimal, i18n-sensitive)

```sql
-- $1 :: TEXT[]  locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($1::TEXT[]) WITH ORDINALITY AS t(locale, ord)
),
per_field_choice AS (
  SELECT
    ii.incident_id,
    ii.field_key,
    ii.field_value,
    row_number() OVER (
      PARTITION BY ii.incident_id, ii.field_key
      ORDER BY lp.ord
    ) AS rn
  FROM incident_i18n ii
  JOIN locale_priority lp ON lp.locale = ii.locale
  WHERE ii.field_key IN ('title', 'short_description', 'location')
),
pivot AS (
  SELECT
    incident_id,
    max(field_value) FILTER (WHERE field_key = 'title' AND rn = 1) AS title,
    max(field_value) FILTER (WHERE field_key = 'short_description' AND rn = 1) AS short_description,
    max(field_value) FILTER (WHERE field_key = 'location' AND rn = 1) AS location
  FROM per_field_choice
  GROUP BY incident_id
)
SELECT
  i.id,
  i.code,
  i.category_code,
  i.start_utc,
  i.end_utc,
  p.title,
  p.short_description,
  p.location
FROM incidents i
LEFT JOIN pivot p ON p.incident_id = i.id
ORDER BY i.start_utc DESC;
```

### Select one incident detail (all fields, i18n-sensitive, API-ready JSON)

```sql
-- $1 :: UUID    incident id
-- $2 :: TEXT[]  locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($2::TEXT[]) WITH ORDINALITY AS t(locale, ord)
),
incident_field_choice AS (
  SELECT
    ii.field_key,
    ii.field_value,
    row_number() OVER (PARTITION BY ii.field_key ORDER BY lp.ord) AS rn
  FROM incident_i18n ii
  JOIN locale_priority lp ON lp.locale = ii.locale
  WHERE ii.incident_id = $1
),
incident_i18n_resolved AS (
  SELECT coalesce(jsonb_object_agg(field_key, field_value) FILTER (WHERE rn = 1), '{}'::jsonb) AS data
  FROM incident_field_choice
),
timeline_item_i18n_choice AS (
  SELECT
    ti.timeline_id,
    ti.field_key,
    ti.field_value,
    row_number() OVER (
      PARTITION BY ti.timeline_id, ti.field_key
      ORDER BY lp.ord
    ) AS rn
  FROM incident_timeline_i18n ti
  JOIN locale_priority lp ON lp.locale = ti.locale
),
timeline_i18n_resolved AS (
  SELECT
    timeline_id,
    coalesce(jsonb_object_agg(field_key, field_value) FILTER (WHERE rn = 1), '{}'::jsonb) AS data
  FROM timeline_item_i18n_choice
  GROUP BY timeline_id
),
timeline_json AS (
  SELECT
    t.incident_id,
    jsonb_agg(
      jsonb_build_object(
        'id', t.id,
        'atUtc', t.at_utc,
        'title', r.data->>'title',
        'details', r.data->>'details'
      )
      ORDER BY t.sort_order, t.at_utc
    ) AS timeline
  FROM incident_timeline t
  LEFT JOIN timeline_i18n_resolved r ON r.timeline_id = t.id
  WHERE t.incident_id = $1
  GROUP BY t.incident_id
)
SELECT jsonb_build_object(
  'id', i.id,
  'code', i.code,
  'categoryCode', i.category_code,
  'startUtc', i.start_utc,
  'endUtc', i.end_utc,
  'title', ir.data->>'title',
  'shortDescription', ir.data->>'short_description',
  'longDescription', ir.data->>'long_description',
  'location', ir.data->>'location',
  'timeline', coalesce(tj.timeline, '[]'::jsonb)
) AS payload
FROM incidents i
CROSS JOIN incident_i18n_resolved ir
LEFT JOIN timeline_json tj ON tj.incident_id = i.id
WHERE i.id = $1;
```

### Select all translations for one incident (role-specific operation)

```sql
-- $1 :: UUID incident id
SELECT
  ii.field_key,
  l.code AS locale,
  ii.field_value
FROM locales l
CROSS JOIN (
  SELECT DISTINCT field_key
  FROM incident_i18n
  WHERE incident_id = $1
) k
LEFT JOIN incident_i18n ii
  ON ii.incident_id = $1
 AND ii.locale = l.code
 AND ii.field_key = k.field_key
ORDER BY k.field_key, l.code;
```

### Update all translations for one incident (replace set, batched, single statement)

```sql
-- $1 :: UUID   incident id
-- $2 :: JSONB  payload array
-- Example element: {"locale":"en","field_key":"title","field_value":"Heating outage"}
WITH incoming AS (
  SELECT
    $1::UUID AS incident_id,
    x.locale::TEXT AS locale,
    x.field_key::TEXT AS field_key,
    x.field_value::TEXT AS field_value
  FROM jsonb_to_recordset($2) AS x(locale TEXT, field_key TEXT, field_value TEXT)
),
deleted AS (
  DELETE FROM incident_i18n ii
  WHERE ii.incident_id = $1
    AND NOT EXISTS (
      SELECT 1
      FROM incoming i
      WHERE i.incident_id = ii.incident_id
        AND i.locale = ii.locale
        AND i.field_key = ii.field_key
    )
  RETURNING 1
)
INSERT INTO incident_i18n (incident_id, locale, field_key, field_value)
SELECT incident_id, locale, field_key, field_value
FROM incoming
ON CONFLICT (incident_id, locale, field_key)
DO UPDATE SET field_value = EXCLUDED.field_value;
```

---

## Maintenances (events) queries

### Select all maintenances/events (list/minimal, i18n-sensitive)

```sql
-- $1 :: TEXT[] locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($1::TEXT[]) WITH ORDINALITY AS t(locale, ord)
),
per_field_choice AS (
  SELECT
    mi.maintenance_id,
    mi.field_key,
    mi.field_value,
    row_number() OVER (
      PARTITION BY mi.maintenance_id, mi.field_key
      ORDER BY lp.ord
    ) AS rn
  FROM maintenance_i18n mi
  JOIN locale_priority lp ON lp.locale = mi.locale
  WHERE mi.field_key IN ('title', 'short_description', 'warning', 'location')
),
pivot AS (
  SELECT
    maintenance_id,
    max(field_value) FILTER (WHERE field_key = 'title' AND rn = 1) AS title,
    max(field_value) FILTER (WHERE field_key = 'short_description' AND rn = 1) AS short_description,
    max(field_value) FILTER (WHERE field_key = 'warning' AND rn = 1) AS warning,
    max(field_value) FILTER (WHERE field_key = 'location' AND rn = 1) AS location
  FROM per_field_choice
  GROUP BY maintenance_id
)
SELECT
  m.id,
  m.code,
  m.category_code,
  m.start_utc,
  m.end_utc,
  m.notified_at_utc,
  p.title,
  p.short_description,
  p.warning,
  p.location
FROM maintenances m
LEFT JOIN pivot p ON p.maintenance_id = m.id
ORDER BY m.start_utc DESC;
```

### Select one maintenance/event detail (all fields, i18n-sensitive, API-ready JSON)

```sql
-- $1 :: UUID   maintenance id
-- $2 :: TEXT[] locale priority, e.g. ARRAY['de','en','fr']
WITH locale_priority AS (
  SELECT locale, ord
  FROM unnest($2::TEXT[]) WITH ORDINALITY AS t(locale, ord)
),
field_choice AS (
  SELECT
    mi.field_key,
    mi.field_value,
    row_number() OVER (PARTITION BY mi.field_key ORDER BY lp.ord) AS rn
  FROM maintenance_i18n mi
  JOIN locale_priority lp ON lp.locale = mi.locale
  WHERE mi.maintenance_id = $1
),
resolved AS (
  SELECT coalesce(jsonb_object_agg(field_key, field_value) FILTER (WHERE rn = 1), '{}'::jsonb) AS data
  FROM field_choice
)
SELECT jsonb_build_object(
  'id', m.id,
  'code', m.code,
  'categoryCode', m.category_code,
  'startUtc', m.start_utc,
  'endUtc', m.end_utc,
  'notifiedAtUtc', m.notified_at_utc,
  'title', r.data->>'title',
  'shortDescription', r.data->>'short_description',
  'longDescription', r.data->>'long_description',
  'warning', r.data->>'warning',
  'location', r.data->>'location'
) AS payload
FROM maintenances m
CROSS JOIN resolved r
WHERE m.id = $1;
```

### Select all translations for one maintenance/event (role-specific operation)

```sql
-- $1 :: UUID maintenance id
SELECT
  mi.field_key,
  l.code AS locale,
  mi.field_value
FROM locales l
CROSS JOIN (
  SELECT DISTINCT field_key
  FROM maintenance_i18n
  WHERE maintenance_id = $1
) k
LEFT JOIN maintenance_i18n mi
  ON mi.maintenance_id = $1
 AND mi.locale = l.code
 AND mi.field_key = k.field_key
ORDER BY k.field_key, l.code;
```

### Update all translations for one maintenance/event (replace set, batched, single statement)

```sql
-- $1 :: UUID   maintenance id
-- $2 :: JSONB  payload array
-- Example element: {"locale":"en","field_key":"title","field_value":"Heating maintenance"}
WITH incoming AS (
  SELECT
    $1::UUID AS maintenance_id,
    x.locale::TEXT AS locale,
    x.field_key::TEXT AS field_key,
    x.field_value::TEXT AS field_value
  FROM jsonb_to_recordset($2) AS x(locale TEXT, field_key TEXT, field_value TEXT)
),
deleted AS (
  DELETE FROM maintenance_i18n mi
  WHERE mi.maintenance_id = $1
    AND NOT EXISTS (
      SELECT 1
      FROM incoming i
      WHERE i.maintenance_id = mi.maintenance_id
        AND i.locale = mi.locale
        AND i.field_key = mi.field_key
    )
  RETURNING 1
)
INSERT INTO maintenance_i18n (maintenance_id, locale, field_key, field_value)
SELECT maintenance_id, locale, field_key, field_value
FROM incoming
ON CONFLICT (maintenance_id, locale, field_key)
DO UPDATE SET field_value = EXCLUDED.field_value;
```

---

## Global translation keys queries

### Select all existing keys and translations with missing values obvious

```sql
SELECT
  k.key_name,
  l.code AS locale,
  v.value,
  (v.value IS NULL) AS is_missing
FROM translation_keys k
CROSS JOIN locales l
LEFT JOIN translation_values v
  ON v.key_name = k.key_name
 AND v.locale = l.code
ORDER BY k.key_name, l.code;
```

### Update all given translations for keys and locales (single batched upsert)

```sql
-- $1 :: JSONB payload array
-- Example element: {"key_name":"nav.events","locale":"de","value":"Ereignisse"}
WITH incoming AS (
  SELECT
    x.key_name::TEXT AS key_name,
    x.locale::TEXT AS locale,
    x.value::TEXT AS value
  FROM jsonb_to_recordset($1) AS x(key_name TEXT, locale TEXT, value TEXT)
)
INSERT INTO translation_values (key_name, locale, value, updated_at)
SELECT key_name, locale, value, now()
FROM incoming
ON CONFLICT (key_name, locale)
DO UPDATE
SET value = EXCLUDED.value,
    updated_at = now();
```

## Why this design serves the requested use cases

- New language: insert into `locales`, then add any translated rows needed.
- User-mode i18n reads: one query with locale-priority array; DB resolves per field.
- Admin-mode all translations: matrix-like queries make gaps visible immediately.
- Add/refactor field: no schema migration for translatable fields (`field_key` row model).
- Missing translations: naturally represented as absent rows; easy to detect with `CROSS JOIN locales`.
