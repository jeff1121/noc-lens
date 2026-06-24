-- noc-lens 初始資料庫結構（對應 data-model.md）

CREATE TABLE IF NOT EXISTS device (
    id           TEXT PRIMARY KEY,
    ip_address   TEXT NOT NULL UNIQUE,
    username     TEXT NOT NULL,
    password_enc BLOB NOT NULL,
    note         TEXT,
    brand        TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS device_group_tag (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS device_group (
    device_id TEXT NOT NULL,
    group_id  TEXT NOT NULL,
    PRIMARY KEY (device_id, group_id),
    FOREIGN KEY (device_id) REFERENCES device(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES device_group_tag(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS status_snapshot (
    id            TEXT PRIMARY KEY,
    device_id     TEXT NOT NULL,
    job_run_id    TEXT,
    collected_at  TEXT NOT NULL,
    status        TEXT NOT NULL,
    error_message TEXT,
    metrics_json  TEXT,
    FOREIGN KEY (device_id) REFERENCES device(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS scheduled_job (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    target_type      TEXT NOT NULL,
    target_id        TEXT NOT NULL,
    schedule_kind    TEXT NOT NULL,
    interval_minutes INTEGER,
    daily_time       TEXT,
    enabled          INTEGER NOT NULL DEFAULT 1,
    created_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS job_run (
    id            TEXT PRIMARY KEY,
    job_id        TEXT NOT NULL,
    started_at    TEXT NOT NULL,
    finished_at   TEXT,
    total         INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (job_id) REFERENCES scheduled_job(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS report (
    id             TEXT PRIMARY KEY,
    title          TEXT NOT NULL,
    scope_json     TEXT NOT NULL,
    summary_md     TEXT NOT NULL,
    generated_at   TEXT NOT NULL,
    model_endpoint TEXT
);

CREATE TABLE IF NOT EXISTS app_setting (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_snapshot_device_time
    ON status_snapshot(device_id, collected_at);
