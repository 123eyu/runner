# #861 — Program status (OSC 7501) from the terminal stream

## Starting point

Jason asked on 2026-10-27 for a codex trio to implement [#861](https://github.com/yicheng47/runner/issues/861). Work only in `/Volumes/samsung/repos/yicheng47/runner/.worktrees/feat-861-program-status`, branch `feat/861-program-status`, based on `origin/main` at `cb059f09`. The root checkout stays on `main`; this worktree uses its own `target/`. The branch has no upstream: push with `git push -u origin feat/861-program-status`.

This mission covers #861 completely: probe/parser and reducer integration. Agent-hook changes, the Settings status-hooks switch, and the Claude Code blocked-versus-PermissionRequest comparison belong to [#1051](https://github.com/yicheng47/runner/issues/1051) and are outside this mission (scope update from Jason, 2026-10-27). The PR says "Closes #861".

## Problem and outcome

Claude Code (2.1.295+) and pi (1.1.0+) probe with `OSC 7501 ; ?` and report program status only when the terminal answers. Runner never answers, so both stay silent. Hooks give agent chats their status today, but they miss an agent started by hand in a terminal tab. After this mission, `runnerd` answers the probe, parses reports from the PTY stream, and feeds them to the session-state reducer: a hand-started `claude` or `pi` in a terminal tab shows Working / Idle / Needs you through existing status visuals, and agent chats with a live hook bridge keep taking status from hooks.

Evidence since the issue was filed: installed versions are Claude Code 2.1.311, pi 1.4.0 and Codex 0.164.0. Codex now emits OSC 7501 unconditionally (no probe); #1008 added a Codex-only raw-byte parser in `runtimes/codex/program_status.rs` that maps `blocked` to a native Needs you wait. That behavior must not regress. Folding it into the shared parser is allowed only if every existing Codex test keeps its meaning; otherwise leave it alone.

## Read first

- The protocol: <https://mitchellh.com/writing/program-status-osc7501> and the specification <https://www.superlogical.com/rex/docs/build/program-status>. Record the exact probe reply, report grammar, keys, terminators and size limits in the spec; do not guess them.
- `crates/runner-terminal/src/terminal/model.rs`: `TerminalModel::feed_output` and the mode 2031 `SchemeSequence::Query` path, which already splits the parse at the query and sends its reply through the ordered events channel. The OSC 7 `CwdReports` scanner is the second precedent.
- `crates/runner-daemon/src/runtimes/codex/{program_status.rs,terminal.rs}`, `session/{pty_runtime.rs,runtime.rs,state/,status.rs}`, `session/manager/output.rs`, and `runner-core/src/protocol/status.rs` (`ObservationSource`).
- `docs/arch/runtime-integration.md` (P0.8), `docs/guide/reference/supported-agents.md`, `docs/tests/regression/{runtimes,missions}.md`.

## Deliverables

1. **Spec first.** Write `docs/features/861-program-status.md` before code: the protocol contract, the state mapping, hook/OSC precedence, which sessions answer the probe, limits and rejection rules, affected guide pages, and a pointer to #1051 for hook changes.
2. **Phase 1, `runner-terminal`.** Answer `OSC 7501 ; ?` at the query's position in the stream: Claude Code treats the DA reply as its no-answer sentinel, so a 7501 reply that lands after a DA reply from the same chunk settles the protocol as unsupported. Parse reports across arbitrary chunk splits with BEL and ST terminators. Apply the spec's size limits and refuse decoded text containing control characters. Unit tests cover split chunks, both terminators, limits, malformed and control-containing input, and reply ordering against a DA query in the same and the next chunk.
3. **Phase 2, reducer.** Reports become an evidence source: `working` → Working, `idle` → Idle, `blocked` → Needs you with `msg` in the visible label, `done`/`error` → the turn outcome. A session with a live hook bridge takes status from hooks; any other session takes it from OSC 7501. Reports never fabricate an unread response, and a stale or late report must not override a newer hook state. Process exit clears program status. Publication reaches direct chats and missions without a mounted pane; check the effect on crew delivery readiness for `blocked` and cover it with a test.
4. **Terminal tabs.** Find how a shell session's status reaches the sidebar and pane header today. Reuse existing status visuals. If showing a terminal tab's agent status needs new UI, stop and ask Jason through Runner rather than inventing it.
5. **Docs.** `runtime-integration.md` names OSC 7501 as the preferred status mechanism for new runtimes; `supported-agents.md` states which agents report it and from which version; add regression rows for the lasting live checks. Protocol additions stay compatible with existing app and CLI clients.

Non-goals: #1051 hook changes, hook comparisons and the Settings switch; #904 notifications; #808 ssh sessions; agent upgrades or global agent configuration.

## Validation and handoffs

Before testing, list the changed behaviors and the check that covers each, per AGENTS.md Test Scope. Automated: `runner-terminal` and `runner-daemon` tests for the parser, reply ordering, reducer mapping, precedence, stale reports, exit clearing and mission readiness; the existing Codex native-attention tests unchanged; `make fmt`, workspace Clippy and the CI nextest command.

`coder` implements and runs checks, then hands the working-tree diff to `reviewer`. `reviewer` reviews code only. After a clean review, `qa` runs these live checks in the development app, with bounded test sessions in fresh `/tmp` folders, inexpensive models, low effort and tiny prompts:

- `claude` started by hand in a terminal tab: Working during a turn, Idle after, Needs you with its message on a permission prompt.
- `pi` started by hand in a terminal tab: Working, Idle and a blocked dialog.
- A Claude Code chat with hooks: status still follows hooks. No hook comparison or shrink recommendation is required.
- A Codex chat: native Needs you on an approval still appears and clears (adapt RT-STATUS for Codex and MIS-WAIT-02 if delivery code changed).

QA does not touch the installed Runner or development sessions it did not create, and archives only its own.

## Authorization

Exactly this three-slot codex trio. No extra missions, slots or internal subagents. Authorized: implementation, tests, the spec, docs, QA's development-app checks above, focused commits on this branch, pushing it, opening a PR against `main`, and driving macOS, Windows and Linux CI green. Then stop. Not authorized: merging, nightlies or releases, archiving the mission, deleting branches or removing worktrees, changing global agent configuration.

History: fold this brief into the implementation commit rather than keeping a brief-only commit; split commits only where each step builds and passes on its own; fold review and CI fixes into the commits they correct and push with `--force-with-lease`. No tool-specific co-author trailers. The PR body carries the review verdict, QA matrix with evidence paths, test results and remaining platform gaps. Hook changes belong to #1051.
