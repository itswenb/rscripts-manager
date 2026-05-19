CREATE TABLE workflow_steps (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  script_path TEXT NOT NULL,
  input_schema JSONB NOT NULL DEFAULT '[]'::jsonb,
  param_schema JSONB NOT NULL DEFAULT '[]'::jsonb,
  output_dir_name TEXT NOT NULL DEFAULT 'outputs',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER workflow_steps_updated_at
  BEFORE UPDATE ON workflow_steps
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at();
