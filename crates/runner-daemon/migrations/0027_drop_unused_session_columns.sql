-- Launch definitions replace overrides; live process identity is pid + agent_command.
ALTER TABLE sessions DROP COLUMN runtime_socket;
ALTER TABLE sessions DROP COLUMN runtime_window;
ALTER TABLE sessions DROP COLUMN runtime_pane;
ALTER TABLE sessions DROP COLUMN runtime_cursor;
ALTER TABLE sessions DROP COLUMN runtime;
ALTER TABLE sessions DROP COLUMN runtime_session;
ALTER TABLE sessions DROP COLUMN agent_model;
ALTER TABLE sessions DROP COLUMN agent_effort;
ALTER TABLE sessions DROP COLUMN runtime_options_json;
