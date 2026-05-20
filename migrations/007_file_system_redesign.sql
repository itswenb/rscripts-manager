-- Redesign file system: user-based folders with a shared public folder
-- Remove project_id dependency, add owner_id and is_public flag

ALTER TABLE file_assets ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE file_assets ADD COLUMN IF NOT EXISTS is_public BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE file_assets ALTER COLUMN project_id DROP NOT NULL;

CREATE INDEX idx_file_assets_owner ON file_assets(owner_id);
CREATE INDEX idx_file_assets_public ON file_assets(is_public) WHERE is_public = true;
