ALTER TABLE event_categories ADD COLUMN IF NOT EXISTS color TEXT NOT NULL DEFAULT '#9aaab1';

UPDATE event_categories
SET color = CASE id
  WHEN 'HEA' THEN '#d73a49'
  WHEN 'ELV' THEN '#0366d6'
  WHEN 'PLB' THEN '#0e8a16'
  WHEN 'ELC' THEN '#f9c513'
  WHEN 'GAR' THEN '#6f42c1'
  WHEN 'PMG' THEN '#005cc5'
  ELSE color
END;
