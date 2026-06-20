-- Merge long_description into short_description before consolidating
UPDATE incident_i18n AS target
SET field_value = target.field_value || '. ' || src.field_value
FROM incident_i18n AS src
WHERE target.incident_id = src.incident_id
  AND target.locale = src.locale
  AND target.field_key = 'short_description'
  AND src.field_key = 'long_description'
  AND src.field_value != '';

DELETE FROM incident_i18n WHERE field_key = 'long_description';
UPDATE incident_i18n SET field_key = 'description' WHERE field_key = 'short_description';

-- Same for maintenances
UPDATE maintenance_i18n AS target
SET field_value = target.field_value || '. ' || src.field_value
FROM maintenance_i18n AS src
WHERE target.maintenance_id = src.maintenance_id
  AND target.locale = src.locale
  AND target.field_key = 'short_description'
  AND src.field_key = 'long_description'
  AND src.field_value != '';

DELETE FROM maintenance_i18n WHERE field_key = 'long_description';
UPDATE maintenance_i18n SET field_key = 'description' WHERE field_key = 'short_description';
