# #968 — Project selector in New chat and New mission

Jason requested a codex trio mission with QA on 2026-10-28. Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/feat-968-project-selector` on `feat/968-project-selector`, based on `origin/main` at `96272824`. The root checkout stays on `main`; this worktree has its own `target/`.

## Problem and outcome

The New chat and New mission dialogs file new work under a project the user can neither see nor change. Add a Project field to both, shown and editable before starting, and keep it in step with the working directory. The spec, [docs/features/968-project-selector.md](../../features/968-project-selector.md), is signed off and wins on behaviour; the design, `runner/desktop/specs/968-project-selector.pen` in `yeelinks_design` (option A `dNbRf`, menu `XkvxJ`, directory states board `ky8Ar`, New mission `xO5YM`), wins on visuals. Do not edit the design file; ask Jason if the design needs to change.

## Partial start in the worktree (uncommitted)

The planning session began implementing inline before handing over. It is a starting point: keep it, rework it or replace it; the reviewer reviews the whole diff.

Done, with tests passing:

- `runner-core` `protocol::project::deepest_folder_of`, the shared longest-ancestor matcher (resolved local paths, lexical host paths with either separator); `runner-daemon` `repo::project::find_for_path` delegates to it, and its `ops::project` tests pass.
- `runner-app/src/surfaces/project_field.rs`: `ProjectChoice` (local rows or host catalog), `ProjectField` (choose, follow, `directory_hint`, `set_choices`, lock, scope), menu options and 5 unit tests.
- `StyledSelect::mono_descriptions`, `Field::warning`, and seven new keys in `en.toml` and `zh-hans.toml`.
- New chat: `StartChatModal.project: ProjectField` replaces `scope`; `StartChatSelection::Project` applies the menu choice to the directory; `follow_start_chat_cwd` runs on working-directory blur, after local Browse, when the remote folder picker closes, and at submit; submission sends the field's scope; `project_start_scope` is gone.

Not done: New chat rendering (the field between Chat name and Working directory, `DirectoryHint::apply` on the directory field, the locked subtitle), its focus order and disabling during submit; all of New mission; the crew page's Start mission using the active project (`crews/list.rs`, `crews/editor.rs`); app-level tests; guide pages. The tree builds with dead-code warnings only.

## Read first

The spec; `surfaces/project_field.rs`; `surfaces/start_chat/{modal,render,controls,launch}.rs`; `surfaces/start_mission.rs` (local and `RemoteMissionForm` paths, `load_remote_mission_catalog`); `surfaces/front_page/client.rs` (`ProjectScope::or_root` for app starts); `surfaces/remote_client/start_chat.rs`; `surfaces/sidebar/menus.rs` (`SidebarMenuAction::NewChat`/`NewMission`); `surfaces/start_chat/terminal.rs` (`new_mission_action`).

## Deliverables

- Both dialogs as the spec's Behaviour section describes: placement, menu (No project, separator, projects in sidebar order with mono paths and a check), initial choice per entry point, the project ↔ directory rules, explicit `Project(id)`/`Root` submission, the pane lock, remote host projects, and the field shown even with no projects.
- New mission's remote form fills its choices when the host catalog arrives (`ProjectField::set_choices`) and sends the chosen host project.
- Two deviations from the design, already in the spec and to be confirmed by Jason on the PR: the pane lock is a disabled select with the subtitle "Chats in a pane file with their tab." rather than a lock icon and tooltip, which `StyledSelect` lacks, and No project uses the app's `clock.svg` for the design's history icon.
- Guide pages named in the spec's Documentation section, English and `zh-CN`, passing `python3 script/check-guide-translations.py`.
- Keep the spec current if implementation settles a detail differently.

Out of scope: creating projects from the dialog, menu search, moving existing work between projects, CLI flag changes, design-file edits.

## Validation and handoffs

Automated, selected for the changed behaviour: the matcher and `ProjectField` tests; New chat and New mission GPUI tests for each entry point's initial project, a menu choice rewriting or keeping the directory, a directory switching the project on blur and after Browse, the outside-project warning, the pane lock, and the submitted scope and directory; the remote form's host projects using the existing remote_client test fakes; then `make fmt`, workspace Clippy and the CI-profile nextest command in `AGENTS.md`.

`coder` implements and hands the working-tree diff to `reviewer` with no commit. After a clean review, `coder` hands the same candidate to `qa` with build and launch steps. `qa` runs these live checks in the development app under the standing authorization in `AGENTS.md` (fresh `/tmp` folders, an inexpensive model, low effort, tiny prompts; archive only what it creates):

1. From the sidebar's New Chat, choose a project and start a chat: it appears under that project.
2. Start a mission with an explicitly chosen project: it appears under that project.
3. Open New chat and New mission from project A's +: A is chosen; change it before starting and the work lands under the new choice.
4. Type or browse a directory inside another project: the selection follows; a directory outside every project shows the amber hint and keeps the choice.
5. An empty pane's New chat shows the locked field.

Windows and a remote computer are left to Jason unless available; `qa` reports them as Blocked rather than skipping silently. QA findings go back through `coder` and a delta review, then `qa` reruns only the affected checks.

## Authorization

Exactly three agents, the codex trio's `coder`, `reviewer` and `qa`; no other missions, internal subagents or delegated agents. After code review and QA are both clean, `coder` commits, pushes `feat/968-project-selector`, opens a PR against `main` and drives CI green on macOS and Windows, then stops. The PR is a clean history on `main`: this brief and the spec fold into the implementation commits rather than landing alone, and review or CI fixes are folded into the commits they correct and pushed with `git push --force-with-lease`. Do not merge, cut a nightly or release, remove the worktree or delete branches. The PR body names the live checks that passed and those left to Jason, and contains no Claude session URL.
