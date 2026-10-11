# 813 — Live-regression input harness

> Tracking issue: [#813](https://github.com/yicheng47/runner/issues/813)
> Priority: P2, 0.15 with [704](./704-session-send.md), which it builds on (moved from 0.13 on 2026-10-07). Platforms: Windows first; works on macOS unchanged.
> Status: draft 2026-10-06; builds on [#704](./704-session-send.md).

## Motivation

On Windows, the QA agent in a Codex crew cannot type into a Runner terminal. It drives the desktop through OpenAI's bundled Computer Use plugin, whose guidance has a "Non-negotiable Windows Automation Safety" section: no automating terminal applications, no running terminal commands through UI automation "directly or indirectly", and no automating "Codex CLI or Codex extensions within Windows apps". User confirmation cannot override these denies. A Runner chat pane is a terminal hosting an agent CLI, so every agent turn, interrupt, resume and draft check is off limits. macOS has no such section, which is why the #777 smoke drove all five runtimes through computer use there.

The 2026-10-06 Windows PC regression (`docs/tests/archive/2026-10-06-pc-full-regression.md`) ended with 73 of 75 five-runtime cells Blocked and the delivery and input rows I1–I6 Blocked for this reason. The crew rightly declined to substitute raw PTY or database writes, because nothing sanctioned them. Computer use is still allowed to look: QA captured agent panes, clicked Runner's own controls and read Settings throughout that run. What is missing is a sanctioned way to put input into a session.

Crews run on Codex for token budget, so the answer cannot be "switch QA to another computer-use tool". A CLI path also costs fewer tokens per step than screenshot-driven typing.

## The split

- **Input** goes through the harness: text, Enter, named keys and unsent drafts, delivered by `runnerd` through the same per-session input path the app's terminal uses.
- **Observation** stays with computer use and the CLI: screenshots of the pane and Runner's chrome, `runner session show`, the mission feed and the runtime's transcript. QA focuses a pane through the sidebar or the harness, not by clicking inside a Codex pane.
- **The physical keyboard path** stays with a person: keyboard → GPUI → terminal encoding, Pinyin composition, clipboard and file paste.

## Scope

### Built on #704

`runner session send` and `runner session wait` from #704 already cover "type a prompt and press Enter when the session is idle" and "block until idle or exit". The harness uses them unchanged for R1, R2, R4, R8–R10 and I1–I4. #704 lands first, or both land in the same mission with #704's phases first.

### Development-only additions

Three abilities #704 deliberately does not offer, because an agent should not be able to interrupt or half-type into another session:

```
runner-dev session key <session> <key>...        # escape, ctrl-c, enter, up, down, left, right, tab, y, n
runner-dev session type <session> <text>         # pastes text with no Enter: an unsent draft
runner-dev session send <session> <text> --busy  # sends while the session is working (queued-message cases)
```

- Keys use the same encoding table the app uses for a focused terminal (`crates/runner-terminal/src/mappings.rs`: legacy Escape or the kitty CSI form, as the pane's mode requires), so the session sees what a real keypress would produce.
- Input enters the session manager's direct-input path (`write_input` behind the delivery gate), never a raw PTY write. It therefore queues behind router delivery, honours the 500 ms input-flush grace, records local cancellation after a successful write and flush, and sets the same draft and local-input state typed input does.
- The commands exist only in development builds (`runner-dev`), or are refused by a release daemon with a clear error. They are limited to the owning user's endpoint, like every other CLI call.

### The scripted run

A script, kept with the full-smoke procedure in `docs/tests/` or a new `scripts/` directory, that drives one runtime through R1–R13 and I1–I6 against a private development home: start a direct chat, send a turn, recall, start a quiet tool and interrupt it with Escape and with Ctrl+C, recover, stop and resume, start a new conversation, approve and deny, hold and clear a draft, and trigger compaction where the runtime supports it. After each step it records `session show` state, the feed cursor and the transcript position, and pauses for QA's screenshot where the checklist asks for one. It writes one JSON result per step so the report can cite them.

### Accounts

The harness does not solve account isolation, and the regression showed it is the other half of the gap. Codex (`CODEX_HOME`), Copilot (`COPILOT_HOME`) and pi (`PI_CODING_AGENT_DIR`, now with a private settings root after F5) can run fully in a private home. Claude Code and Antigravity cannot use an existing login from a private home without touching the real configuration.

**Decision for Jason:** cover Claude Code and Antigravity by a one-time login into the private development home (kept between runs, never copied from the real one), or leave them Blocked on Windows and cover them on macOS only. Recommended: the one-time private login, since Claude Code is the runtime most likely to differ on Windows (Git Bash hooks, F2).

### Docs

Update `docs/tests/full-smoke-test.md`: which rows the harness proves on each platform, the remaining manual rows (keyboard path, IME, clipboard and paste), and that on Windows QA observes through computer use but sends input only through the harness.

## Out of scope

- Bypassing or replacing the Computer Use policy, or adding another desktop-automation tool to the crew.
- A user-facing automation or "remote typing" feature. The additions stay development-only.
- Hardware and setup rows the regression also left Blocked: a second monitor or scale, a second Windows account, logoff and shutdown, signed installers and real upgrades.

## macOS

No change to the release app on macOS. The development-only commands go through the same cross-platform input path, so they also work on macOS, where a crew may use them instead of computer-use typing to save tokens. The macOS full smoke can keep computer-use input; the harness is an alternative there, not a replacement.

## Implementation phases

1. #704's backend and CLI, if not already landed.
2. `session key`, `session type` and `--busy` in the development CLI and daemon, refused by release builds, with tests that each one reaches the session through the direct-input path and the delivery gate.
3. The scripted run for Codex first, then Copilot and pi, then Claude Code and Antigravity once the account decision is made.
4. A Windows run of R1–R13 and I1–I6 for at least Codex and Claude Code, recorded in `docs/tests/`, and the full-smoke doc update.

## Verification

- [ ] Each harness command reaches the session through `write_input` behind the delivery gate: input queues behind an in-flight router delivery, and a draft typed with `session type` holds router delivery as a typed draft does.
- [ ] `session key escape` during a tool interrupts it in Codex, Claude Code and pi on Windows, and status reports the interruption as it would for a real keypress.
- [ ] A release build has no `session key` or `session type`, and a release daemon refuses the development-only requests.
- [ ] The scripted run completes R1–R13 and I1–I6 for Codex and Claude Code on Windows with one result file per step, and QA's screenshots match the recorded states.
- [ ] The macOS release app behaves the same, and the development commands work there too.

## References

- Computer Use guidance: `~/.codex/plugins/cache/openai-bundled/computer-use/<version>/docs/guidance.md`, section "Non-negotiable Windows Automation Safety".
- Windows regression report and QA checklist: `docs/tests/archive/2026-10-06-pc-full-regression.md`.
- Input path: `crates/runner-daemon/src/session/manager/terminal.rs` (`write_input`), `session/manager/output.rs` (delivery gate), `crates/runner-terminal/src/mappings.rs` (key encoding).
- Related: [#704](./704-session-send.md) (session send and wait), #777 (runtime adapter smoke on macOS), [#797](./797-hook-status-ipc.md) (hook status IPC), #645 (runnerd).
