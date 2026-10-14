# 876 — computer-use QA of the OpenCode runtime

Run a native macOS computer-use test of [PR #876](https://github.com/yicheng47/runner/pull/876), OpenCode v2 runtime support from an external fork. Jason asked on 2026-10-14 for a mission that runs this test and is scoped to OpenCode only. This is test-and-report: no code changes, commits, pushes or PR comments. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-592-opencode-runtime`, local branch `feat/592-opencode-runtime` at the PR head `ffa1ead2` (CI green on macOS and Windows). The only untracked files are this brief and the run record you write. Do not touch the root checkout or other worktrees, and do not share another worktree's `target/`.

## Read first

- `AGENTS.md`: Test Scope and Full Smoke Tests.
- `docs/tests/full-smoke-test.md`: candidate identity, macOS native control (the `.app` wrapper), evidence and cleanup. Reuse those sections; do not adopt its runtime matrix.
- The PR's own records: `docs/features/592-opencode-runtime.md`, `docs/tests/592-opencode-runtime.md` and `docs/tests/runs/2026-10-13-mac.md`. The contributor's computer-use tooling failed, so every GUI check is still open. Their backend evidence came from a pre-rebase candidate and is not evidence for `ffa1ead2`.
- `docs/tests/regression/{runtimes,missions,settings}.md` for the case templates below.

## Prerequisite: OpenCode v2

The PR targets OpenCode v2 (`@opencode/cli`; the contributor used 2.0.26, and 2.0.28 is the latest). As of brief time this Mac had only v1.18.31 at `~/.opencode/bin/opencode`, and Jason is installing v2. Build first. Before any agent check, find the v2 binary by absolute path and record its `--version`. If it is still absent, ask the human through Runner, mark the agent checks Blocked and continue with the checks that do not need it. Never install, upgrade or sign in to anything yourself.

## Isolation

Run the candidate app under a private HOME so that neither Jason's Runner data nor his OpenCode v1 data is touched:

- Use a short canonical scratch root such as `/private/tmp/oc876.XXXXXX`, with HOME at `<root>/h`, private `XDG_CONFIG_HOME`/`XDG_DATA_HOME`/`XDG_STATE_HOME`/`XDG_CACHE_HOME`, `ZDOTDIR` and `TMPDIR`, and evidence at `<root>/evidence`. The daemon socket `<HOME>/Library/Application Support/com.wycstudios.runner-dev/runnerd.sock` must stay under 104 bytes.
- The private `ZDOTDIR` profile puts the v2 binary's directory first on PATH. Do not add `~/.opencode/bin` (that is v1).
- Build with `cargo build -p runner-cli -p runner-app` in the worktree. Do not use `make run`, because it runs under the real HOME. Launch the wrapper `.app` as the procedure describes, with that environment, `NO_COLOR` unset and `RUNNER_CREW_ID`/`RUNNER_MISSION_ID`/`RUNNER_HANDLE`/`RUNNER_EVENT_LOG` removed. Then bind computer use to it. Verify with the candidate CLI (absolute path, same environment) that `status --json` reports the private socket.
- Use only OpenCode's built-in free models. Never connect a provider, sign in, or copy anything from `~/.local/share/opencode`, `~/.config/opencode` or `~/.opencode`. Before and after the run, record the hashes and mtimes of `~/.config/opencode/opencode.json` and `~/.local/share/opencode/opencode.db*` to show they did not change.
- Never touch Jason's production Runner, his development data under the real HOME, or any pre-existing session. Never open a Runner database.

## Cases (OpenCode only)

Drive every case through the real UI with computer use, and capture screenshots plus structured state from the candidate CLI. Verify control with a harmless action first. If computer use cannot bind, report that at once and mark the UI checks Blocked; CLI output does not substitute for pixels. Use one child folder per chat or mission, tiny prompts, and an explicit model and effort where OpenCode offers them.

1. **Settings.** SET-AGENT-01 covers OpenCode detection, version and path, and the brand mark in light and dark themes. SET-AGENT-02 sets the OpenCode executable override to the v2 path, tries a missing path, and restores. SET-SKILL-01 and SET-SKILL-03 use the private XDG and `.agents/skills` roots. SET-MCP-01 and SET-MCP-02 cover the OpenCode JSONC catalog, an edit that preserves comments and unrelated keys, and a copy to and from another client.
2. **Direct chat lifecycle, RT-LIFE-01 to 09.** Cover an ACK and a blind recall. Check that the requested model and effort match Runner's picker or panel and OpenCode's own footer. Run a visible foreground `sleep 30` and capture the Working indicator. Interrupt it with OpenCode's advertised key and recover with `RECOVERED`. Stop and resume with the visible restored history, a blind recall and the same native key. Then `/new` with the NEW marker, and stop/resume it.
3. **Native session switching.** Adapt RT-NAV-01: after `/new`, use OpenCode's own session switcher to return to ORIGINAL and complete a blind recall. `session show` must report ORIGINAL's key again. RT-NAV-04 covers two OpenCode chats in the same cwd that keep their keys and recalls independent across stop/resume.
4. **Roles and crews.** RT-START-01 covers a single-slot OpenCode crew's bounded append goal in a never-trusted folder. RT-START-02 covers an OpenCode role with a persona, including stop/resume. MIS-CONFIG-01 covers slot model/effort overrides and the effective `--auto` mission permission, seen in the process argv and in OpenCode's UI.
5. **Mission bus.** Use a two-slot OpenCode crew. MIS-MSG-01 sends an addressed human message while the recipient is Working and again while it is Idle, with read/ACK both times. MIS-ASK-01 has the worker run `ask_lead`, the lead escalate with `ask_human`, and the card answered natively. Throughout, capture the side panel's Working/Ready indicators.

Out of scope: other runtimes, IME, general terminal and workspace behavior, Windows, installation and update, and the full regression suite. Record any of these that you notice as observations, not cases.

## Findings

Report each failure to the human as soon as it is established: case, steps, expected and actual, and evidence paths. To attribute it, repeat the minimal step in plain `opencode` v2 under the same private environment outside Runner. A failure that also happens there is native, not Runner's. Keep first failures even when a retry passes. Continue with the cases a failure does not block.

## Record and cleanup

Write `docs/tests/runs/2026-10-14-macos.md` in the worktree in the suite's run-record format, adapting case IDs as `<ID> (opencode)`. Include candidate identity (HEAD, app and CLI hashes, OpenCode version), the isolation proof, the result matrix and findings, and leave it uncommitted. Clean up as `full-smoke-test.md` says: stop and archive your chats and missions, stop the wrapper and private daemon, unregister the wrapper, and confirm with `ps` that nothing of yours survives. Keep the scratch root, evidence, worktree and native histories.

## Authorization

Jason's request authorizes the bounded OpenCode chats, roles, crews and missions above, inside the private Runner namespace, using OpenCode free models, plus stopping and resuming your own sessions. Nothing is committed or pushed, the PR is not commented on, and no nightly or release is cut. The only slot is `qa`.

Final Runner handoff to the human: the overall verdict (Passed, Failed or Incomplete), the result matrix, findings with evidence paths, the record path, and a short summary that Jason can post on the PR. Then stand by.
