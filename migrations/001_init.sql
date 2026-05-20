CREATE TABLE IF NOT EXISTS admin (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS pipeline_nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    script_path TEXT NOT NULL,
    params_schema TEXT NOT NULL DEFAULT '[]',
    inputs_schema TEXT NOT NULL DEFAULT '[]',
    outputs_schema TEXT NOT NULL DEFAULT '[]',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS project_flows (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS project_flow_steps (
    id TEXT PRIMARY KEY,
    flow_id TEXT NOT NULL REFERENCES project_flows(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL REFERENCES pipeline_nodes(id),
    step_order INTEGER NOT NULL,
    param_values TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS flow_runs (
    id TEXT PRIMARY KEY,
    flow_id TEXT NOT NULL REFERENCES project_flows(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    current_step INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    finished_at DATETIME
);

CREATE TABLE IF NOT EXISTS step_runs (
    id TEXT PRIMARY KEY,
    flow_run_id TEXT NOT NULL REFERENCES flow_runs(id) ON DELETE CASCADE,
    step_order INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    slurm_job_id TEXT,
    stdout TEXT,
    stderr TEXT,
    started_at DATETIME,
    finished_at DATETIME
);
