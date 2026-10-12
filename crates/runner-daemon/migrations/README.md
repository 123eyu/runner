# Migrations

Runner's SQLite schema, as numbered SQL migrations that `runnerd` applies when it starts, and [`schema.sql`](schema.sql), a snapshot of the whole schema after the latest one.

## The snapshot

`schema.sql` is the database after every migration here, from SQLite's own record of it: each table followed by its indexes, in name order, including the bookkeeping tables `_migrations` and `_app_state`. Read it for a table's current shape, and the migrations for how it got there.

It is generated and never edited by hand. The test `db::tests::schema_snapshot_matches_migrations` builds a fresh database the way `runnerd` opens one and fails when the file differs. Regenerate it with:

```sh
RUNNER_UPDATE_GOLDEN=1 cargo test -p runner-daemon schema_snapshot
```

SQLite keeps each table's original `CREATE TABLE` text and splices every `ALTER TABLE … ADD COLUMN` into it on one line, so the snapshot lays each table out again with one column or table constraint per line, keeping a trailing `--` comment with its column. The text is otherwise SQLite's: a table renamed by a migration keeps the quotes SQLite adds (`"sessions"`, `"roles"`), and comments are whatever the creating migration wrote, even when a later migration made them stale.

## Adding a migration

1. Name it `NNNN_snake_case_summary.sql` with the next number. A migration that has shipped is never edited, renumbered or deleted; fix it forward with a new one.
2. Open the file with a comment saying what changes and why, and what `NULL` means in any new nullable column.
3. Register it in `MIGRATIONS` in [`src/db/migrations.rs`](../src/db/migrations.rs), with a short entry in the comment above the list.
4. Regenerate `schema.sql` in the same commit.
5. When the migration moves or rewrites data, test it against a database at the previous version with `run_migrations_up_to`, as the 0023 tests in `src/db/tests.rs` do.

## How migrations run

- `runnerd` applies every pending migration at start, each in its own `IMMEDIATE` transaction together with its `_migrations` row, so a failure rolls back and the next start retries it. There are no down migrations.
- Foreign keys are on for the whole run: `init_connection` sets `PRAGMA foreign_keys = ON` before any migration, and SQLite ignores that pragma inside a transaction.
- A data step that needs Rust runs inside the migration's transaction, as `backfill_0014_nodes` does. A conversion that needs files or the running daemon, such as the mission roster conversion in `src/daemon/boot.rs`, runs at `runnerd` start instead and is deleted two releases after it ships.
- Seed data is not a migration; it lives in `src/db/seed.rs`.

## DDL rules

- **Prefer additive changes:** `ALTER TABLE … ADD COLUMN` with a nullable column or a `NOT NULL DEFAULT`, saying what `NULL` means.
- **Rename in place.** `ALTER TABLE … RENAME TO` and `RENAME COLUMN` keep the data and update references in other tables, as 0023 does with `PRAGMA legacy_alter_table = OFF`.
- **Never drop or rebuild a table that another table references.** Changing a column's type, its `NOT NULL` or its foreign key needs a rebuild, and with foreign keys on inside the transaction, dropping the old table runs its `ON DELETE` actions on every child row. Rebuilding `missions` would set every `sessions.mission_id` to `NULL`; rebuilding `sessions` would delete all of `session_attention`. Renaming the old table out of the way first does not help, because SQLite repoints the children at the renamed table, which is then dropped. Both were checked with SQLite 3.51 on 2026-10-12. Such a rebuild first needs the runner to apply that migration with foreign keys turned off before its transaction opens and `PRAGMA foreign_key_check` passing before it commits; add that to the runner in its own change. Tables that nothing references (today `slots` and `session_attention`) can be rebuilt the way 0007 rebuilt `sessions` before anything referenced it: create `<table>_new`, copy, drop, rename, and recreate the indexes.
- **Every foreign key states its `ON DELETE`** (`CASCADE`, `SET NULL` or `RESTRICT`), as every existing one does.
- **Types.** Ids are `TEXT`, ULIDs for rows Runner creates. Timestamps are `TEXT` in RFC 3339, except where a table already counts epoch milliseconds in an `INTEGER` (`session_attention.unread_since`). Booleans are `INTEGER` 0 or 1. Structured values are JSON in `TEXT` columns named `*_json` (`nodes.layout` predates this), with a version field when the shape is expected to change.
