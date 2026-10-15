# 853 — the empty start surface

Implement [P2 #853](https://github.com/yicheng47/runner/issues/853). Jason asked on 2026-10-14 for a codex trio mission that ends in an open PR, not a merge. There is no spec: the issue is the record and wins over this brief on any detail.

Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-853-empty-start-surface`, on `feat/853-empty-start-surface`, cut from `origin/main` at `bf2cb087` (the design is `9c76efce`). Its tip is this brief. Do not create another branch or checkout, touch the root checkout or another worktree, or share another worktree's target directory. Other crews may be working in other `.worktrees/`; treat them as another machine's checkout. If main moves and you need it, rebase onto `origin/main`; never merge main into the branch.

## Read first

- `AGENTS.md`: Worktrees, Test Scope and Crew Missions.
- `gh issue view 853`: scope, design notes and verification.
- **The design**, exported to this worktree's ignored `target/design-ref/`: `start-surface-{dark,light}.png`, `empty-pane-split-{dark,light}.png` (1600 × 1040 frames, window at 80,70), and `cmp-start-surface@2x.png`, `cmp-empty-pane-start@2x.png`. Do not open or edit `.pen` files. Measurements from the components:
  - **Start surface**: column 480 px wide, centred on the work card with 48 px bottom padding, gap 28 between the mark, the heading block and the actions card. Mark 44 px, `brand-mark.svg` geometry in the accent, drawn with a thin stroke (about 2.5 px at 44 px; add a light-weight asset beside `BRAND_MARK` if the existing one reads heavy). Title "Start something new" 28 px semibold `text`; subtitle "Choose how you want to work." 14 px `muted`; gap 8. Actions card: `panel`, 1 px `border`, radius 12, padding 6, row gap 2. Rows: 84 px tall, radius 8, padding 16 × 20, gap 16; lucide icon 22 px (`message-square-plus` in `accent`, `terminal` and `users` in `muted`); label 15 px semibold, description 13 px `muted`, gap 5; shortcut badge 28 px tall, at least 58 px wide, `raised` fill, 1 px `border_strong`, radius 6, 12 px medium `muted`. Copy: New chat / "Work with an agent or a saved role", New terminal / "Run commands in a shell", New mission / "Work with a crew of agents".
  - **Background**: the work card's `bg` with an accent wash, a linear gradient from the top: accent at alpha 0x12 (about 7 %) fading to transparent at 60 % of the height. Derive it from `theme::accent()` so every theme tracks it.
  - **Empty pane in a split**: column 280 px, gap 12; "Open in this pane" 13 px medium `muted`; card `panel`, 1 px `border`, radius 10, padding 4, gap 2; rows 40 px, radius 6, padding 0 10 0 12, gap 10, 16 px icon, 13 px medium label, badge `raised` with 1 px `border_strong`, radius 5, padding 3 × 7, 11 px medium `muted`. New chat and New terminal only.
  - The design draws rest states only. Hover uses the house row hover. No new keyboard focus order: the shortcuts are the keyboard path.
- `crates/runner-app/src/surfaces/panes.rs`: the no-tab branch of `render_active_tab` (~329, keep the daemon banner above it), the empty-pane branch (~2508), `empty_pane_action_label` (~2721) and its test.
- `keymap.rs` ~142 (`new-chat`, `new-terminal`, `new-mission`), `effective_binding`, `format_combo`.
- `surfaces/start_chat.rs`: `new_terminal_action`, `new_mission_action`, `focused_empty_chat_pane`, `open_pane_chat_modal`, `spawn_terminal_in_pane`.
- The close path: `close_tab` in `main.rs` ~282, `request_close_single_pane_tab` and `prepare_tab_close` in `surfaces/chat.rs` ~2096 and ~2148, `PendingTabClose` and `apply_tab_rows` in `main.rs` ~428 and ~1086, `TabSet` in `pane_layout.rs` ~662, `tab_close_neighbour` in `surfaces/sidebar/shortcuts.rs` ~76, `sidebar/archive.rs` ~141.

## Deliverable

1. **Start surface.** One reusable element shown when no tab is active, and when the active tab's only pane is empty. It replaces "No direct chats yet" and "No active tab". Each row runs the same action as its shortcut (`NewChat`, `NewTerminal`, `NewMission`), so the row and the key never diverge. The badge reads the effective binding through `effective_binding` + `format_combo`, so a rebind shows the user's own combo and Windows reads `Ctrl+N`; an unbound action shows no badge. No sidebar row is selected while it shows. The sidebar keeps its New chat entry.
2. **Empty pane in a split.** The smaller element in an empty pane of a multi-pane tab. New chat opens the start chat modal for that pane (today's `open_pane_chat_modal`); New terminal spawns a shell into that pane, the path ⌘T takes for the focused empty pane. A click on an unfocused pane's row acts on that pane. The pane header keeps "Empty".
3. **Landing rule.** Archiving the chat in the active tab, closing the active terminal tab, and ⌘W on the active tab all land on the start surface instead of the neighbouring tab. All of them pass through `prepare_tab_close`, so change the rule there: the pending close clears the active tab rather than activating a neighbour. `TabSet::replace_rows` today falls back to the first tab whenever the active id is gone or unset, so a node reload would leave the start surface a moment later. Give `TabSet` a cleared state that survives reloads until something activates a tab, while the first load at launch behaves as today. Archiving one chat of a split tab still leaves its pane empty; archiving a chat that is not on screen changes nothing. Remove `tab_close_neighbour`, `neighbour_tab_id` and their tests if nothing else calls them.
4. **Tests** beside the existing ones:
   - the badge text for default, rebound, Windows-formatted and unbound bindings, replacing `empty_pane_offers_only_the_chat_entry_point`;
   - which element renders for no tab, a single empty pane, an empty pane in a split, and a filled tab;
   - the landing rule: closing the active tab leaves no active tab, and a later `replace_rows` keeps it so;
   - closing an inactive tab leaves the active one alone;
   - launch still activates a tab;
   - a headless layout check that the start surface column stays 480 px and centred at 900 and 1440 px window widths (`debug_selector` + `VisualTestContext::simulate_resize`).
   Gate any import or helper used only by `cfg(unix)` tests with `#[cfg(unix)]`.

Out of scope: `runnerd`, the protocol, the CLI and the database; the start chat modal and Start mission dialog themselves; which tab is active at launch; `.pen` files; README screenshots.

## Validation

Run each, log it to a file and read the exit code from the command itself; never pipe a gate through `tail` or `grep`:

- `cargo nextest run --locked -p runner-app --no-fail-fast --cargo-profile ci`
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, then again with `--features updater`
- `cargo fmt --all --check`
- `git diff --check`

## QA

After clean review, `qa` runs one native macOS pass on the development app with computer use, through the `Runner Smoke Dev.app` wrapper in `docs/tests/full-smoke-test.md` (setup, candidate identity, evidence, cleanup). **No real agent launches and no accounts**: use terminals, shell sessions and the modals only, and cancel the modals rather than starting anything. Checks: close the last tab and see the start surface in dark and light; each row opens its flow (start chat modal, a terminal, Start mission dialog); close the active terminal tab while other tabs exist and land on the start surface, which stays put after a few seconds of activity; rebind New terminal in Settings → Keyboard shortcuts and see the new badge, then restore it. Mark Blocked any check that needs an agent chat (archiving the active chat, an empty pane in a split chat tab); Jason runs those. QA never touches Jason's production app or pre-existing development sessions, reports Blocked rather than stopping a development app it did not start, and stops the development daemon at the end. Native Windows is unverified.

## Boundaries and authorization

The coder and reviewer never run the development app; only `qa` does, as above. No slot starts, stops, restarts or types into Jason's production Runner, its chats or its missions; this mission runs there. No extra agents, crews or subagents.

The reviewer waits for an explicit handoff, then reviews the whole working-tree diff against the issue, the PNGs and this brief, must-fix findings first with file:line pointers. It checks in particular: the landing rule cannot be undone by a reload; an inactive-tab archive changes nothing; the rows dispatch the same actions as the keys; the badges follow the keymap; the pane variant never offers New mission; nothing changes outside `runner-app`. Iterate until the reviewer posts `NO REMAINING MUST-FIX ISSUES`, then QA as above.

After clean review and QA, Jason authorizes:

- **Commit**: one commit on top of current `origin/main` with this brief folded in, such as `feat(ui): redesign the empty start surface`. No co-author trailers.
- **Push** with `git push -u origin feat/853-empty-start-surface`, and **open the PR** with `gh pr create --base main`: `Closes #853`, a summary, checks with exit codes, the QA matrix, what waits for Jason (the Blocked checks, Windows, and a visual pass of both themes), no agent session links.
- **Watch CI** with `gh pr checks <n> --watch` until nothing is pending on macOS and Windows. Fold each fix into the commit (`--amend`), have the reviewer check it, push with `git push --force-with-lease`.

**Do not merge**, delete the branch or worktree, or cut a nightly or release. The final handoff goes to everyone through Runner: the PR URL and CI result, changed files, checks with exit codes, the QA matrix, what is unverified, and the reviewer's verdict. Then all slots stand by.
