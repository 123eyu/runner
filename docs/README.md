# Runner Docs

Project markdown lives here. Keep repo docs close to the code; the memory repo holds durable cross-session context, not project decisions — those belong in these files.

## Layout

- [`roadmap.md`](./roadmap.md) — where the project is: the latest release, what has landed since, the release plan mirrored from the GitHub milestones, and the decisions shaping the next releases. Dated; update it when an issue moves milestone, a release is cut, or a mission lands.
- [`guide/`](./guide/README.md) — the user guide, with [authoring conventions](./guide-authoring.md) for keeping it aligned with shipped behavior.
- [`arch/`](./arch/) — architecture references: how the system works today.
- [`product/`](./product/) — product vision and direction: why we're building this, which surfaces matter.
- [`features/`](./features/README.md) — feature specs, `{number}-{slug}.md`. Since 2026-09-01 the number **is** the spec's GitHub tracking issue — file the issue first, then name the spec after it.
- [`impls/`](./impls/README.md) — implementation plans for concrete build slices, named after their feature since 2026-09-01; mission briefs in `impls/briefs/`.
- [`tests/`](./tests/) — [maintained live regression cases](./tests/regression/README.md), the [execution procedure](./tests/full-smoke-test.md), feature validation plans and historical records. New run results go in `tests/runs/YYYY-MM-DD-<platform>.md`, separate from the suite.
- [`tech/`](./tech/README.md) — deep dives on the libraries Runner builds on, pinned to the dependency versions in `Cargo.lock`, written for reading the code.

`archive/` is for history that still helps: design rationale behind the implementation, migrations and compatibility constraints, or validation evidence. A completed or abandoned document does not automatically need an archive copy. Preserve lasting decisions in `arch/` and deferred scope in the tracking issue; remove obsolete or duplicate specs and plans after checking references and protecting uncommitted edits. Git history retains the old text. Distant features normally need only an issue until development starts.

Keep test evidence and task history: promote lasting live checks into `tests/regression/`, or state why none apply in the record. Maintained cases describe current behavior; dated runs stay in `tests/runs/` and historical feature records in `tests/archive/`. Retained specs and plans move to their respective `archive/` at the release sweep; documents still in the parent directory should describe active work or clearly identify work merged since the last release.
