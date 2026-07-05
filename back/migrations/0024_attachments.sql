CREATE TABLE attachments (
  id          UUID PRIMARY KEY,
  item_type   TEXT NOT NULL CHECK (item_type IN ('incident', 'maintenance', 'project')),
  item_id     UUID NOT NULL,
  file_name   TEXT NOT NULL,
  mime_type   TEXT NOT NULL,
  size_bytes  BIGINT NOT NULL,
  storage_key TEXT NOT NULL,
  uploaded_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at  TIMESTAMPTZ
);

CREATE INDEX attachments_active_idx ON attachments(item_type, item_id) WHERE deleted_at IS NULL;
