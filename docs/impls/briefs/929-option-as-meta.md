# #929 / #1003 — Option as Meta key on macOS

Jason requested this mission on 2026-10-23: the codex duo crew, where `reviewer` reviews the diff and then runs the live checks. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-929-option-as-meta` on `fix/929-option-as-meta`, based on `origin/main` at `348cbad5`. The root checkout stays on `main`.

## Problem

Every Option chord in a terminal pane is encoded as Meta. `terminal_key_route` sends any `alt` chord down the raw path and `encode_key` turns it into ESC plus a character, so on layouts where Option types `@ [ ] { } | \ ~` (German, Spanish, Nordic, Swiss and others) those characters cannot be typed. Option+L on German sends `ESC l` instead of `@`; #1003 is a user who cannot type `@` in Claude Code. gpui already reports both characters: `key = "l"`, `key_char = "@"`.

## Behavior: follow Ghostty's `macos-option-as-alt`

A new setting, **Settings → Terminal → Option as Meta key**, with five values:

- **Automatic** (default, also for existing settings files): Meta on `com.apple.keylayout.US`, `com.apple.keylayout.USInternational` (Ghostty's list, `src/input/keyboard.zig`) and `com.apple.keylayout.ABC`; Off on every other layout. Runner adds ABC because its Option layer is the U.S. one and it is what this Mac uses. Verify the U.S. International ID on macOS and include the real one (likely `…USInternational-PC`). Read the layout through gpui's `keyboard_layout().id()`. A layout switch takes effect on the next keypress.
- **Both**: Option is always Meta (today's behavior).
- **Left only** / **Right only**: only that Option key is Meta; the other types characters. gpui's `Modifiers` has one `alt` flag, so read the device-dependent bits (`NX_DEVICELALTKEYMASK` 0x20, `NX_DEVICERALTKEYMASK` 0x40) from `NSApplication.currentEvent` during key-down. gpui dispatches key-down synchronously inside the AppKit handler, so that event is the current one. Keep this macOS-only.
- **Off**: Option always types characters.

Rules in every mode:

1. When Option is not Meta for a keypress and `key_char` is printable (no control characters), do not take the raw path. Let the event propagate so gpui's macOS handler passes it to the input context and the character reaches the PTY through the terminal's text input (`replace_text_in_range`), dead keys included.
2. An Option chord with no printable character (Option+Enter, Option+Backspace, Option+arrows, function keys, any Ctrl+Option chord) is Meta in every mode. #855's Option+←/→ word motion must keep working.
3. Meta sends ESC plus the character the key types without Option: gpui's `key` with shift applied as today, never `key_char`. Today Option+5 on German sends `ESC [`, which starts a CSI. It must send `ESC 5`, as Ghostty does. The kitty-protocol path keeps its current encoding.
4. `swallows_option_copy` (#387) is unchanged. Windows is unchanged and its row is hidden; AltGr on Windows is out of scope.

## Design

PNG exports are in this worktree's ignored `target/design-ref/`: `option-row.png` (the row after Cursor style, value Automatic), `option-menu-open.png` (the menu with a description per option, using the select's described-option menu like Quit behavior's) and `behavior-note.png`. Their source is `runner/desktop/specs/929-option-as-alt.pen` in the private design repo. Do not open or edit `.pen` files. The PNGs decide layout and copy; this brief decides behavior. Translate the new strings for 简体中文.

## Read first

- `crates/runner-app/src/terminal_ime.rs`: `terminal_key_route`, `swallows_option_copy`, the `EntityInputHandler` impl.
- `crates/runner-terminal/src/mappings.rs`: `encode_key` and the #855 macOS word-motion branch.
- The three key-down callers: `surfaces/chat.rs` `on_key_down`, `surfaces/mission_workspace/input.rs` and `surfaces/agent_update.rs`.
- `app_settings.rs` `AppSettings`; `surfaces/settings_page.rs` `render_terminal_settings`, `settings_select`, `SettingsSelection`; `ui/select.rs` `SelectOption` descriptions.
- gpui-pre-macos 0.3.7: `window.rs` `handle_key_event` (the callback runs before the input context) and `events.rs` (`key` vs `key_char`); `keyboard.rs` for the layout ID.

## Deliverables

- The setting, persisted and defaulting to Automatic, with the Settings row and menu as designed (macOS only).
- The routing and encoding rules above, shared by all three callers, with unit tests: German Option+L (`key_char` `@`) is text in Automatic/Off and Meta in Both; Left only treats right Option as text; ABC/U.S. Automatic keeps Option+B as `ESC b`; Option+5 Meta sends `ESC 5`; Option+Enter/Backspace/arrows stay Meta in Off; existing #855 and #387 tests still pass.
- Guide updates: `docs/guide/guides/appearance.md` (the Settings → Terminal paragraph) and the Option note in `docs/guide/reference/keyboard-shortcuts.md`.
- A test record at `docs/tests/929-option-as-meta.md`, written by `reviewer` from the live run.

## Validation

Before testing, list the changed behaviors and the check that covers each, per [Test Scope](../../../AGENTS.md#test-scope). Automated, in this worktree's own target directory: `cargo nextest run --locked -p runner-app -p runner-terminal --cargo-profile ci`, the runner-daemon terminal tests if `encode_key` changes their bytes, workspace Clippy with `-D warnings` (`--profile ci`), `cargo fmt --all --check`, `git diff --check`.

Live checks: `reviewer` runs them after its code review is clean, on the revision `coder` freezes and hands over (HEAD and the SHA-256 of `git diff HEAD`). Jason's request overrides AGENTS.md's "reviewers do not run the app" for this mission. Only one slot runs Cargo at a time. Use the development app as in [Full smoke test procedure](../../tests/full-smoke-test.md): its `.app` wrapper, development CLI, scratch folders under `/tmp`, evidence and cleanup sections. Jason authorized Claude Code test chats for this mission despite the 2026-10-16 runtime note: inexpensive model, low effort, nothing submitted. Codex chats follow the procedure's trust-entry rule.

Switching the keyboard layout is authorized: record the current input sources first, enable German only for the run, and restore the original selection and enabled list at the end. If computer use cannot switch layouts or press the right Option key (try a synthesized CGEvent with keycode 61 and the right-Option device flag), ask the human through Runner and mark that check Blocked.

1. German, Automatic, zsh prompt: Option+L `@`, Option+5 `[`, Option+6 `]`, Option+7 `|`, Shift+Option+7 `\`; Option+←/→ move by word.
2. German, Automatic: `@` in a Claude Code composer opens its file mention (#1003); `@` in a Codex composer. Nothing submitted.
3. ABC, Automatic: Option+B/F move by word at zsh (Meta kept).
4. Modes: Off on ABC types `∫` for Option+B; Both on German makes Option+L Meta again; Left only on German: left Option+L is Meta, right Option+L types `@`. The choice survives an app relaunch.
5. The row and open menu match the PNGs in dark and light.

Keep screenshots and the received bytes in the evidence directory.

## Authorization and handoff

Exactly the two codex duo slots; no other agents, missions or internal subagents. `coder` implements, commits, pushes this branch and opens one PR against `main` with `Fixes #929` and `Fixes #1003`, after review is clean and `reviewer` has reported the live verdict. Fold this brief into the implementation commit. Fold review and CI fixes into the commits they correct, pushed with `git push --force-with-lease`. No tool-specific co-author trailers. A Failed or Blocked live check goes into the PR description as it is. Drive macOS and Windows CI green, then stop and report through Runner with the PR URL, final SHA and results. Do not merge, release, archive the mission, or remove the branch or worktree. When waiting for a teammate, finish the turn; Runner pushes the next message.
