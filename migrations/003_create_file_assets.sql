CREATE TABLE file_assets (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  parent_id UUID REFERENCES file_assets(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  is_directory BOOLEAN NOT NULL DEFAULT false,
  size_bytes BIGINT NOT NULL DEFAULT 0,
  mime_type TEXT,
  storage_path TEXT NOT NULL,
  deleted_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_file_assets_project ON file_assets(project_id);
CREATE INDEX idx_file_assets_parent ON file_assets(parent_id);
CREATE UNIQUE INDEX idx_file_assets_unique_name ON file_assets(project_id, parent_id, name) WHERE deleted_at IS NULL;

CREATE TRIGGER file_assets_updated_at
  BEFORE UPDATE ON file_assets
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at();
