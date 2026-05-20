ALTER TABLE pipeline_runs ADD COLUMN input_files jsonb NOT NULL DEFAULT '{}';
ALTER TABLE pipeline_runs ADD COLUMN param_overrides jsonb NOT NULL DEFAULT '{}';
