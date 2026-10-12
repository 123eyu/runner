-- Record the immutable launch inputs. NULL means conversion could not compose a launch;
-- the Rust step fills every resumable row in this migration's transaction.
ALTER TABLE sessions ADD COLUMN launch_definition_json TEXT;
