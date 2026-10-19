# 887 — Implement the approved Active sidebar

Jason approved the final sidebar and requested a Codex duo to first implement the code on 2026-10-19. Use the existing `coder` + `reviewer` crew. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-887-active-sidebar`, branch `feat/887-active-sidebar`, created from `origin/main` at `cb12a9ce92fba6add522078617f59f6903ddb411`. Do not create another branch/worktree or share a Cargo target directory. The branch initially tracks origin/main; that is not permission to push to main.

## Read first

- `AGENTS.md` and `docs/features/887-active-sidebar.md` in this worktree.
- The approved root-checkout Pencil file `/Users/jason/repos/yicheng47/runner/design/specs/887-active-sidebar-view.pen`, through Pencil only. Final screen `XzJHS`; sidebar `BSxqq`; default `YxSEl`; empty `sqqIk`; shortcuts `W5o1T`; decisions `osxwG`.
- Snapshot PNGs exported after approval at `/private/tmp/runner-887-design/{XzJHS,YxSEl,sqqIk,W5o1T,osxwG}.png`. Use these for visual reference even if Pencil is being used for another project. All `.pen` editing belongs in the root checkout and is outside this coding mission.
- [#887](https://github.com/yicheng47/runner/issues/887). Its older group order, row layout and open questions are superseded by the spec and approved design.
- `crates/runner-app/src/surfaces/sidebar/{mod,view,rows,elements,shortcuts}.rs`, `sidebar_logic.rs`, the existing settings persistence and usage/footer code, and the session/tab/mission projections they use.

## Implement

Use the spec as the acceptance contract. Add the remembered icon toggle next to Search and render Active with **Chats → Missions → Terminals**, counts and hidden empty groups. Include Error → Needs you → Working → Unread, then recent activity with deterministic ties. Retain a selected row while it is being read; remove an otherwise-ineligible row after navigating away. No timer.

Rows are individual sessions, with exact tab/pane activation, a title plus project-relative location/compact cwd, and a trailing status glyph. Roll missions up once, without listing their member sessions again. Preserve existing window-ownership routing. Number shortcuts follow the same visible order; the reference is chats 1–5, missions 6–7, terminals 8–9. Preserve context menus and remove Active drag/drop targets without changing Projects organization or behavior.

Match the final shared sidebar chrome and footer, including the conditional app-update icon beside Runner, Workspace controls, dividers, separate Settings gear, and **only Codex and Claude** usage at bottom. Usage items are **left-aligned, gap 10 px**, transparent until hover, using existing live weekly values/popover and availability semantics. Do not hardcode the screenshot percentages, remove another runtime's support, or reproduce the broken Antigravity image from earlier drafts.

Use current terminal Busy/activity evidence. #861 and #904 Phase 2 are open and outside scope: no OSC parser, shell integration, foreground-process detection, or notification feature. Drawer shells remain excluded. Preserve localization, themes, sidebar resize and platform-specific window chrome. The unrelated `836-web-remote-control.pen` changes in the root checkout belong to other work.

Update the guide pages named by the spec. Keep this scoped; no unrelated refactors or speculative abstractions.

## Review and checks

Coder owns code and tests; reviewer reviews only after an explicit Runner handoff. Review the complete working tree, including new files, against the spec and screenshots. Use Runner messages with branch, changed paths, source identity, checks/results and concrete file:line findings. Iterate until reviewer reports `NO REMAINING MUST-FIX ISSUES`. Reviewer must verify activation, list/shortcut consistency, status retention, mission deduplication, persistence and platform/localization handling, as well as visible design fidelity. Neither slot starts additional agents or uses native subagents.

Before testing, list the changed behaviors and selected checks with reasons. Run:

```sh
cargo test --locked -p runner-app --profile ci
cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings
cargo clippy --locked --workspace --all-targets --profile ci --features updater -- -D warnings
cargo fmt --all --check
git diff --check
```

Add relevant crate tests only when that crate changes. Record exact results in `docs/tests/887-active-sidebar.md`; do not claim native macOS/Windows or full regression coverage. This is code and review first: do not launch the native app, operate existing development/installed sessions, add QA, or alter global agent settings. Name the selected later native checks from the spec for Jason's next step.

## Authorization and stopping point

The requested first pass is implementation plus review with changes left uncommitted. Do not commit implementation, push, open a PR, merge, publish, cut a nightly/release, or delete a branch/worktree without further explicit authorization. This overrides the usual PR-ending mission default. Jason explicitly authorized the launch-only setup commit on 2026-10-19: it covers only the initial spec, its index entry and this brief, and does not enable PR mode. If later authorized to publish, fold the setup brief/spec into clean implementation commits, keep history on top of main, fold review/CI fixes into their owning commits, and drive macOS/Windows CI green before returning an open PR. Never merge or release without an explicit ask.

Finish by reporting the reviewed worktree/branch, what changed, check results, reviewer verdict, remaining limitations and the short native-validation checklist through Runner. Then stop and wait.


## Later authorization — 2026-10-19

After native feedback, Jason requested “Fixed this, and then pr.” This authorizes correcting Active to include running chats and missions (including idle agents) and exclude stopped selected items without unread/error attention, then committing, pushing and opening a PR after the working-tree review is clean. Fold the launch-only setup brief/spec into the implementation commit, preserve history on top of main, fold review/CI fixes into their owning commit and drive macOS/Windows CI green. The earlier uncommitted stopping point describes the initial pass and is superseded for this phase. Merge, release, branch/worktree deletion and further QA remain unauthorized; Jason owns native testing.
