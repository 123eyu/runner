-- Missions keep optional crew provenance (NULL means an empty start), without a foreign key.
-- The app owns dependent-row cleanup; foreign keys refuse deletion rather than cascade.
-- Applied with foreign keys off before the transaction; checked before commit.

CREATE TABLE missions_new (
    id TEXT PRIMARY KEY,
    crew_id TEXT,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    goal_override TEXT,
    cwd TEXT,
    started_at TEXT NOT NULL,
    stopped_at TEXT,
    pinned_at TEXT,
    archived_at TEXT,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL
);
INSERT INTO missions_new (id, crew_id, title, status, goal_override, cwd, started_at, stopped_at, pinned_at, archived_at, project_id)
SELECT id, crew_id, title, status, goal_override, cwd, started_at, stopped_at, pinned_at, archived_at, project_id FROM missions;
DROP TABLE missions;
ALTER TABLE missions_new RENAME TO missions;

CREATE INDEX idx_missions_project ON missions(project_id);

CREATE TABLE slots_new (
    id TEXT PRIMARY KEY,
    crew_id TEXT NOT NULL REFERENCES crews(id) ON DELETE RESTRICT,
    role_id TEXT NOT NULL REFERENCES "roles"(id) ON DELETE RESTRICT,
    slot_handle TEXT NOT NULL,
    position INTEGER NOT NULL,
    lead INTEGER NOT NULL DEFAULT 0,
    added_at TEXT NOT NULL,
    runtime_override TEXT,
    model_override TEXT,
    effort_override TEXT,
    runtime_options_json TEXT,
    UNIQUE (crew_id, slot_handle),
    UNIQUE (crew_id, position)
);
INSERT INTO slots_new (id, crew_id, role_id, slot_handle, position, lead, added_at, runtime_override, model_override, effort_override, runtime_options_json)
SELECT id, crew_id, role_id, slot_handle, position, lead, added_at, runtime_override, model_override, effort_override, runtime_options_json FROM slots;
DROP TABLE slots;
ALTER TABLE slots_new RENAME TO slots;

CREATE TABLE sessions_new (
    id TEXT PRIMARY KEY,
    mission_id TEXT REFERENCES missions(id) ON DELETE SET NULL,
    role_id TEXT REFERENCES "roles"(id) ON DELETE RESTRICT,
    slot_id TEXT,
    cwd TEXT,
    status TEXT NOT NULL,
    pid INTEGER,
    started_at TEXT,
    stopped_at TEXT,
    agent_session_key TEXT,
    archived_at TEXT,
    title TEXT,
    pinned_at TEXT,
    agent_runtime TEXT,
    agent_command TEXT,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    last_cols INTEGER,
    last_rows INTEGER,
    resume_on_launch INTEGER NOT NULL DEFAULT 0,
    live_title TEXT,
    launch_definition_json TEXT
);
INSERT INTO sessions_new (id, mission_id, role_id, slot_id, cwd, status, pid, started_at, stopped_at, agent_session_key, archived_at, title, pinned_at, agent_runtime, agent_command, project_id, last_cols, last_rows, resume_on_launch, live_title, launch_definition_json)
SELECT id, mission_id, role_id, slot_id, cwd, status, pid, started_at, stopped_at, agent_session_key, archived_at, title, pinned_at, agent_runtime, agent_command, project_id, last_cols, last_rows, resume_on_launch, live_title, launch_definition_json FROM sessions;
DROP TABLE sessions;
ALTER TABLE sessions_new RENAME TO sessions;

CREATE INDEX idx_sessions_project ON sessions(project_id);

CREATE TABLE session_attention_new (
    session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE RESTRICT,
    unread_since INTEGER,
    error_acknowledged_at TEXT
);
INSERT INTO session_attention_new (session_id, unread_since, error_acknowledged_at)
SELECT session_id, unread_since, error_acknowledged_at FROM session_attention;
DROP TABLE session_attention;
ALTER TABLE session_attention_new RENAME TO session_attention;
