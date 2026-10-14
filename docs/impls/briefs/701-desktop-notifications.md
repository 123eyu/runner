# 701 — desktop notifications when an agent needs you

Implement [P2 #701](https://github.com/yicheng47/runner/issues/701). Jason asked on 2026-10-13 for a codex trio mission that ends in an open PR, not a merge.

Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-701-desktop-notifications`, on `feat/701-desktop-notifications`, cut from `origin/main` at `8c47a317` (the spec and design). Its tip is this brief together with the implementation plan. Do not create another branch or checkout, touch the root checkout or another worktree, or share another worktree's target directory. Another crew may be working in `.worktrees/feat-562-add-remove-roles`; treat it as another machine's checkout. If main moves and you need it, rebase onto `origin/main`; never merge main into the branch.

## Read first

- `AGENTS.md`, including Worktrees, Test Scope and Crew Missions.
- **`docs/features/701-desktop-notifications.md`**, the spec. It wins over the plan, this brief and the issue on any detail.
- **`docs/impls/701-desktop-notifications.md`**, the plan: what exists today with file and line pointers, the decisions, where each change lands, and the three commits. Follow its decisions. If the code proves one wrong, change the plan on this branch and say why in the handoff.
- **The design**, exported as PNGs in this worktree's ignored `target/design-ref/`:
  - `settings-notifications.png`: the Settings → Notifications pane.
  - `notification-content-dark.png` and `notification-content-light.png`: what each trigger puts in the title and body. The system draws the banner itself; these only show the text.

  Match the pane’s layout, hierarchy, spacing and copy with the existing theme tokens and settings components. Jason finalized the design on 2026-10-13 and committed/pushed it on main as `ce2b8f81`: all Chats choices on; four independent Missions choices off, always visible; no mission master. `settings-notifications-light.png` shows the light pane. The former missions-on PNG is obsolete and removed. Coder does not edit or commit the `.pen`; rebase onto main including `ce2b8f81` after clean review. Preserve the refreshed PNGs; implementation stays in this worktree.

- **The notification API in `gpui-pre` 0.3.7**, read-only, under `~/.cargo/registry/src/*/`:
  - `gpui-pre-0.3.7/src/app.rs` 1620–1663 and `platform.rs` 441;
  - `gpui-pre-macos-0.3.7/src/system_notifications.rs` and `gpui-pre-windows-0.3.7/src/system_notifications.rs`;
  - the test helpers in `gpui-pre-0.3.7/src/app/test_context.rs` 440–455.

## Deliverables

The three commits in the plan, each building and passing its tests on its own:

1. **The setting and its pane.**
   - `NotificationSettings` in `app_settings.rs`, with the spec's defaults.
   - `SettingsPane::Notifications` after Appearance, with the bell icon.
   - The pane matching `settings-notifications.png`, including the System row's button and its Windows variant.
   - `app_identifier` made public, and `set_app_identity` at startup.
2. **Direct chats.**
   - `crates/runner-app/src/notifications.rs` with the pure rules and the controller.
   - Status triggers, content, visibility, withdrawal, click routing, and dismissal on quit.
   - The view-change hooks in `surfaces/windowing.rs`.
3. **Missions.**
   - Slot triggers, with the slot's handle fetched only when that slot posts.
   - One whole-mission Busy → Idle trigger, controlled by the Goes idle switch.
   - The question subscription, and `MissionWorkspace::focus_feed`.
   - Mission visibility and routing, and the hooks in `surfaces/mission_workspace/input.rs`.

**Tests**: the unit and `TestAppContext` controller tests listed under Verification in the plan, in the commit whose code they cover. Gate any import or helper used only by `cfg(unix)` tests with `#[cfg(unix)]`; Windows Clippy fails on unused ones.

**Test record**: `docs/tests/701-desktop-notifications.md`. It lists the changed behaviours, the selected checks with a reason for each, the automated results with exit codes, and QA's matrix with evidence.

**Out of scope**:
- `runnerd`, the client protocol, the database and the CLI;
- sound, action buttons and any app or mission master switch; only the independent event switches are in scope;
- Dock or taskbar attention, and badges;
- README screenshots; the explicit design-first revision above is the sole root-checkout exception.

## Validation

Run each, log it to a file, and read the exit code from the command itself; never pipe a gate through `tail` or `grep`:

- `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, then again with `--features updater`
- `cargo fmt --all --check`
- `git diff --check`

## QA

Jason originally authorized one native development-app pass on 2026-10-13. At 07:36 UTC he stopped live QA and took the remaining checks for the PR build himself. QA cleans up its fixtures and wrapper, archives the leftover `aa` mission under explicit authorization, records what it verified, marks remaining cases "Not run: Jason tests the PR build", and releases the candidate. No further live QA is authorized; after the final switch change and clean review, coder proceeds directly to the authorized PR steps. The following procedure and case descriptions document the original pass and the human checks still needed.

**Procedure.** Follow `docs/tests/full-smoke-test.md` for setup, candidate identity, agent configuration isolation, evidence and cleanup. Notifications need an app bundle, so run the candidate through that procedure's `Runner Smoke Dev.app` wrapper (bundle id `com.wycstudios.runner.smoke-dev`). Never use `script/bundle-mac --dev`: it carries the production bundle id.

**Scope.** Bounded test chats and one bounded test mission with real agents on Jason's existing sign-ins. Use a fresh project folder under `/tmp` and tiny prompts. A Claude Code chat asked to run a harmless shell command such as `ls` produces an approval wait.

**macOS settings.** QA may accept the wrapper's notification permission prompt, and may turn the wrapper's own entry in System Settings → Notifications off and on again. QA never changes the installed Runner's entry or any other system setting.

**If macOS refuses the wrapper.** Ad-hoc sign it with `codesign --force --sign - "$smoke_bundle"` and record that.

Selected checks:

- **NOTIF-1, the background post.** With Runner behind another app, a direct chat reaches an approval wait. One banner appears, titled with the chat title, with the body "Approval needed · <project>". Runner does not take focus. Note whether the first notification after the permission prompt is lost, as the plan's Risks expect.
- **NOTIF-2, clicking.** Clicking the banner brings Runner forward on the right window, tab and pane. Repeat with the chat in a second window (⇧⌘N).
- **NOTIF-3, foreground suppression and withdrawal.** With any Runner window in front, a wait in another tab/window posts nothing. Moving Runner to the background does not replay that wait. A fresh background edge posts; switching to its target withdraws it from Notification Center, and clearing the wait withdraws it too.
- **NOTIF-4, visible means silent.** In a split tab of the focused window, an approval wait in the pane without keyboard focus posts nothing.
- **NOTIF-5, replacement and finishing.** A second trigger for the same chat replaces its notification; Notification Center holds one entry. With the default switches and Runner in the background, a fresh completion posts "Finished · <project>". Turning Chats → Finishes a turn off withdraws that notification and suppresses fresh completions; re-enabling does not replay one already observed.
- **NOTIF-6, switches.**
  - Turning off Chats → Needs your input withdraws the visible notification and stops new ones.
  - All four mission rows are always visible and off by default. With all off, slot waits/errors, questions and mission idle post nothing. Turn each on independently: only that kind can post, without replaying older states.
  - With Missions → Goes idle on and Runner in the background, one whole-mission Busy → Idle edge posts with mission title and "Every crew member is idle"; a single slot finishing while siblings remain busy posts nothing, and repeated Idle does not repost. Clicking idle opens Feed. Becoming Busy withdraws it.
  - Turning any mission event switch off withdraws only that kind and stops new ones. The other choices stay as saved. Turning Goes idle off withdraws its notification; re-enabling never replays an old idle edge.
  - Restore the defaults afterwards.
- **NOTIF-7, a mission question.** Turn Missions → Asks you a question and Needs your input on first. With Runner in the background, in a disposable crew’s mission, the lead's `ask_human` posts "@lead asked you · <mission>" with the question as the body. Clicking it opens the mission workspace on the feed, and answering withdraws it. A slot's approval wait posts "@handle · <mission>" with the body "Approval needed", and clicking it opens that slot's tab.
- **NOTIF-8, permission.** With the wrapper turned off in System Settings → Notifications, nothing shows. Turning it back on brings notifications back without restarting Runner. The pane's Open System Settings button lands on the system's notification settings.
- **NOTIF-9, quit.** Quitting the wrapper withdraws its notifications.
- **Regression**, because the change adds hooks to these paths or a pane to this list:
  - SET-AGENT-01: open every Settings pane, including the new one;
  - WS-PANE-02: switching the focused pane;
  - MIS-LIFE-02: switching between the Feed and slot tabs;
  - MIS-ASK-01: the ask and answer flow;

  all from `docs/tests/regression/`.

**Limits.**
- QA creates and archives only its own test chats, roles, crews and missions, and never touches Jason's production app or its sessions.
- If another development app, or a development daemon owned by someone else, is running, QA reports Blocked and does not stop it.
- Native Windows is unavailable. Record toasts, click routing and taskbar grouping under the development identity as unverified for Jason.

## Boundaries

The coder and reviewer never run the development app; only `qa` does, as above. No slot starts, stops, restarts or types into Jason's production Runner, its chats or its missions; this mission itself runs there. Never run a Runner binary while guessing its path, and never launch a second production Runner. No extra agents, crews or subagents beyond QA's own bounded test chats and mission in the development app.

## Review, verification and authorization

The coder owns implementation, tests and fixes. The reviewer waits for an explicit Runner handoff, then reviews the whole branch diff against the spec, the plan, the PNGs and this brief. Must-fix findings come first, with file:line pointers. It checks in particular:

- **No panic from window access.** No entity is read or updated while it is leased: the view-change hooks defer, and the controller runs from `cx.observe` on the store, never inside the store's or a window's update.
- **No blocking call on a refresh.** `session_list` runs only when a slot posts, and the diff runs only when `revisions.activity` or `revisions.missions` changed.
- **Trigger edges.** The first snapshot is a baseline, shells never post, and mission slots never post finished turns; aggregate mission idle requires the Goes idle switch. Any focused Runner window suppresses all new posts. A refresh posts at most one notification per session, by priority.
- **Every withdrawal path from the spec**, including quit, which must not block or slow quitting.
- **Foreground suppression** for any Runner tab/pane/route/window, and target visibility for withdrawal in split panes, mission slot tabs and Feed.
- **Settings.** An old settings file without the field loads the defaults, and the pane matches `settings-notifications.png`.
- **Identity.** `set_app_identity` uses `com.wycstudios.runner` for release builds and `com.wycstudios.runner-dev` for debug builds.
- **No change** to `runnerd`, the protocol, the database or the CLI.

Iterate through Runner until the reviewer posts `NO REMAINING MUST-FIX ISSUES`. Keep QA’s partial record and explicit cleanup/release; no further live QA after Jason’s latest instruction. Then proceed to the authorized PR steps below.

After the clean review and QA, Jason authorizes:

- **Commits.** Rebase the branch into the three commits above on top of current `origin/main`, each one building and passing on its own. Fold this brief, the plan and the spec correction into the first commit, and the test record into the last. Use imperative subjects that name the change, such as `feat(ui): add the Notifications settings pane`, `feat(ui): notify when a direct chat needs you` and `feat(mission): notify about mission slots and questions`. No co-author trailers.
- **Push** with `git push -u origin feat/701-desktop-notifications`.
- **Open the PR** with `gh pr create --base main`. The body has:
  - `Closes #701` and a summary;
  - the checks with exit codes, and the QA matrix;
  - what waits for Jason: NOTIF-1 through NOTIF-9 banner, click, permission and quit checks, all four mission switches, and Windows toasts, clicks and taskbar grouping;
  - no agent session links.
- **Watch CI** with `gh pr checks <n> --watch` until nothing is pending, on both macOS and Windows. Fold any fix into the commit it corrects (`git commit --fixup` with an autosquash rebase), have the reviewer check it, and push with `git push --force-with-lease`.

**Do not merge**, delete the branch or worktree, or cut a nightly or release.

The final handoff goes to everyone through Runner:

- the PR URL and CI result;
- the changed files;
- the checks with exit codes;
- the QA matrix;
- any deviation from the spec or plan;
- what is unverified;
- the reviewer's verdict.

Then all slots stand by.
