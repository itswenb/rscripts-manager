CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL DEFAULT ''
);

INSERT OR IGNORE INTO settings (key, value) VALUES
    ('runtime.mode', 'host'),
    ('runtime.singularity_image_dir', ''),
    ('runtime.singularity_image', ''),
    ('runtime.module_name', '');
