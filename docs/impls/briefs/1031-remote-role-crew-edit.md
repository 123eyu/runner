# #1031 — Roles and crews over the companion protocol

Jason authorized this codex duo mission on 2026-10-26. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-1031-remote-role-crew-edit` on `feat/1031-remote-role-crew-edit`, based on `origin/main` at `3da540053f4364ea90f3437f3f41d12fd4c6f8c6`, with this worktree's own `target/`. The root checkout stays on main.

## Problem and outcome

A paired client can start chats and missions from a host's roles and crews but cannot change them: the companion protocol reads them only through `catalog` and has no write. The iPhone app (#912) needs to manage them. Add companion requests that read, create, edit and delete a host's roles, crews and slots, served by `runnerd`'s companion adapter through the same operations the desktop's Roles and Crews pages use. Protocol and host only: no desktop UI, no CLI command.

The spec, [`docs/features/1031-remote-role-crew-edit.md`](../../features/1031-remote-role-crew-edit.md), is authoritative for the requests, their fields, results, errors and privacy rules. Where this brief and the spec differ, the spec wins; raise a real conflict with the lead rather than guessing.

## Read first

- The spec above, then [the archived #836 spec's Remote actions](../../features/archive/836-web-remote-control.md#remote-actions) for the rules the existing actions follow (one operation two callers, features, fixed diagnostics, wire details).
- `crates/runner-core/src/companion/actions.rs` (`ActionRequest`, `NAMES`, `field_problem`, `Catalog`), `messages.rs` (`ClientMessage::decode_value`), `mod.rs` (`wire_enum!`, `ErrorCode`) and `tests.rs` with `fixtures/`.
- `crates/runner-daemon/src/companion/actions.rs` (`DaemonCore::action`, `catalog`, the `deleteProject` refusal mapping), `core.rs` (`Failure`, `failed`), `mod.rs` (`FEATURES`) and `tests.rs`; `crates/runner-daemon/src/remote/client/action_tests.rs` for driving requests against a real host.
- The operations: `ops::role` (`role_create`, `role_update`, `role_delete`, `ensure_delete_allowed`), `ops::crew`, `ops::slot`, and the event names their `protocol/api.rs` wrappers emit.
- The desktop editors as the reference for what each request sends: `runner-app/src/surfaces/roles/{create,edit}.rs`, `roles/logic.rs` (`permission_modes`, `role_permission_mode`), `crews/{create,editor,add_slot,popup}.rs`.

## Deliverables

1. **Wire types** in `runner-core`: the 13 requests in the spec added to `ActionRequest`, `NAMES` and `field_problem`; result types for `role` and `crew`; a camelCase permission-mode wire enum with an `Unrecognized` fallback, like the existing `wire_enum!` types. A golden file for every new request and for the `role` and `crew` results.
2. **Host adapter** in `runnerd`: each request mapped to its operation in `DaemonCore::action`, each name in `FEATURES`. Updates build the operation's input with every editable field set (role `env` left unchanged). `permissionMode` is read with `runtime_metadata`'s permission inference, as `role_permission_mode` does; share that function from `runner-core` rather than duplicating it if both crates need it. A null `command` on create uses `Runtime::command()`. Errors as the spec lists: `notFound`, `badRequest`, fixed-message refusals, generic `failed`. Each write emits the events its client-protocol wrapper emits (`role/changed`, `crew/changed`, `slot/changed`; `crew_delete` emits both crew and slot).
3. **Guide**: update the trust paragraph in `docs/guide/guides/remote-access.md` as the spec's Guide section says. Keep the remote window's read-only description accurate; do not describe controls that do not exist.

## Easy to get wrong

- **The environment never crosses.** No `env` in a result, fixture, error or log. Test it with a role whose `env` is set.
- **Fixed diagnostics.** Refusals from the operations are free-text `anyhow` errors that name crews, chats and missions. Do not forward them. Prefer checking the refusal's condition with the same repository queries, or a typed error from the operation, over matching message strings; the existing `deleteProject` string match is not a pattern to copy. `Failure` holds a `&'static str`, so each refusal gets its own fixed message.
- **Events.** The adapter calls `ops` directly, not the `api.rs` wrappers, so it emits the events itself. Missing them leaves the host desktop's open Roles or Crews page stale.
- **Whole-record updates.** No double-option on the wire; nullable fields mean null. `updateRole` has no `handle`.
- **Redaction.** `ActionRequest`'s `Debug` stays redacted now that requests carry system prompts; do not log request bodies.
- **`runner call`.** The new names stay unreachable through it; the existing CLI test iterates `NAMES`, so keep it passing rather than special-casing.

## Validation

Name each changed behavior and the check that covers it before running checks. Selected:

- `runner-core` companion tests: round-trip of every new fixture, `badRequest` naming each missing or mistyped field, `unknownRequest` unchanged.
- `runnerd` companion and remote action tests against a real host database: success and stored effect per request, events emitted, reads returning what writes stored, `permissionMode` inference, each refusal's code and message, unknown ids, and no `env` in any result.
- The CLI's `generic_call_has_no_remote_requests`.
- `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, the affected crates' tests with `cargo nextest run --locked --cargo-profile ci`, and `git diff --check`.

No live checks: nothing a person sees or operates in the running app changes, and the in-process tests exercise the whole path. Neither slot runs the development app.

## Roles and handoffs

The coder implements and hands to the reviewer with the commit SHA and the checks run. The reviewer reviews the full diff against the spec and this brief, with particular attention to the Easy to get wrong list, and returns findings or a clean review. Iterate until clean. After a clean review, the coder pushes, opens the PR and drives macOS and Windows CI green; the reviewer confirms the final pushed SHA is the one it reviewed.

## Authorization

Jason authorized this two-slot crew (coder, reviewer), implementation, validation, commits, pushing this branch, and opening one PR against `main` right after a clean review. Shape the history as AGENTS.md's Crew Missions says: fold this brief and the spec into the implementation commits, a subject that names the change, every commit building and passing its tests; fold review and CI fixes into the commits they correct and push with `git push --force-with-lease`, never plain force. No co-author trailers and no Claude session URLs in the PR body. Rebase onto `origin/main` only when AGENTS.md requires it; never merge main into the branch.

Stop at an open PR with green CI on both platforms and report the PR URL, final SHA, a short summary and the validation results. Jason reviews and merges. Do not merge, cut a nightly or release, archive the mission, delete branches or remove worktrees. No internal subagents, extra missions or other delegated agents. When waiting for a teammate, finish your turn and rely on Runner's pushed inbox notification; do not poll or watch the feed.
