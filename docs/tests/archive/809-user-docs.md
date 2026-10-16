# 809 — User guide validation

Documentation and generation checks for [#809](https://github.com/yicheng47/runner/issues/809), performed on macOS on 2026-10-15 in `.worktrees/docs-809-user-guide`, based on `origin/main` at `3f5394c2`. The latest published stable release was verified as `v0.16.2` (`59a8c1e0`). This is a documentation validation record, not a native smoke or full regression pass.

## Scope selected before validation

The change adds 17 user pages and an index, a CLI-help generator, make targets, CI freshness checks and an authoring rule. It does not change application or CLI behavior. The checks below cover guide navigation and formatting, the new generator's output and failure behavior, and the CLI used as the reference source. No live regression cases were selected because no runtime, terminal or UI behavior changed. The issue's native fresh-install walkthrough requirement is retained as a validation gap below.

| Check | Reason | Result |
| --- | --- | --- |
| Parse all pages with `markdown-it-py`, with tables enabled | Check one title, a leading summary, no raw HTML, and usable Markdown | Passed: 18 pages rendered locally |
| Resolve parsed relative links and heading fragments | Keep GitHub and website navigation usable | Passed: 115 relative links and anchors; no link leaves the published guide |
| Compare index links with the page tree | Ensure every public page is published once | Passed: all 17 content pages, no duplicates or missing pages |
| GitHub Markdown API in GFM mode | Verify actual GitHub handling of headings, tables, code and alerts | Passed for all 18 files; expected heading, table, fenced-code and warning-alert structures present |
| `make guide-cli-check` | Match the committed reference to the branch's built binary | Passed: 62 sections, including command overview, agent guide and every public command help path |
| Regenerate to a temporary file | Check deterministic output without modifying the committed page | Passed: byte-for-byte match, including with a different parent terminal width and forced-color environment |
| Add a stale line to a temporary copy and run `--check` | Prove that stale documentation fails the required gate | Passed: exit 1, diagnostic and diff; stale copy unchanged |
| Run `--check` against a missing temporary output | Prevent check mode from silently generating or accepting a missing page | Passed: exit 1; no file created |
| Parse CI YAML and inspect both platform jobs | Verify the new gate is wired on macOS and Windows | Passed: both build the CLI and invoke the generator with `--check`; PowerShell propagates build failure |
| `cargo test --locked -p runner-cli --profile ci -j 6` | Check the CLI parser, help, documented exits and existing command fixtures used as sources | Passed: 205 tests; 2 existing manual socket benchmarks ignored |
| `cargo clippy --locked -p runner-cli --all-targets --profile ci -j 6 -- -D warnings` | Validate the CLI build used for generation | Passed |
| `cargo fmt --all --check` and `git diff --check` | Preserve repository formatting | Passed |

The initial GitHub rendering request was rejected by automatic approval review because repository visibility was unknown. `gh repo view` established that the repository is public and the submitted payload consists only of newly authored guide pages for the requested public PR; the same rendering request was then approved and passed. No private source, user configuration or conversation history was submitted.

The first staged whitespace check found trailing alignment spaces in the binary's help output. The generator was changed to strip trailing spaces per line while preserving indentation, then regeneration, freshness, stale/missing-output and whitespace checks were repeated. The CLI's text and options are unchanged.

## Source audit

The CLI source, keymap, launch dialogs, settings page and runtime metadata in the worktree have no differences from `v0.16.2`. The guide uses that release behavior rather than unreleased issue proposals.

| Guide area | Sources checked |
| --- | --- |
| Installation and platform requirements | `README.md`, `script/bundle-mac`, `script/windows/runner.iss`, `docs/arch/windows.md`, Settings → Agents and Command line code |
| First chat and first mission | `surfaces/start_chat.rs`, `surfaces/start_mission.rs`, daemon seed, `examples/pair-coding/` and mission operations |
| Roles, crews and projects | Protocol models, crew/slot operations and project directory resolution |
| Sessions, quit and archive | Session and mission operations, General settings, daemon lifecycle documentation |
| Mission membership and agent-driven workflow | CLI command definitions, `help agents`, mission start/add/remove operations and their fixtures |
| Windows shortcuts and focus scopes | `crates/runner-app/src/keymap.rs`, including `windows_default`, fixed dialog bindings and terminal copy propagation |
| Appearance and notifications | Settings panes and platform-gated material controls |
| MCP and skills | Settings panes, runtime metadata, native config adapters, catalog fixtures and `runtime_ui.rs` captions |
| Runtime limitations | README capability table and archived runtime validation records; gaps retained for OpenCode, Grok and unverified Windows integrations |

Provider setup links use the URLs from Runner's runtime catalog. No provider installation commands or hard-coded model recommendations were added. Examples use current CLI flags, with PowerShell-specific syntax where needed.

## Limits and remaining acceptance

- A fresh installation through a completed first chat and first mission was not exercised on either platform. No development or installed GUI was launched, and no real agent session was created for this change. A source audit and CLI tests do not establish that native walkthrough.
- Native Windows settings, shortcuts and commands were checked against Windows-specific source, not executed on a Windows desktop locally. The PR's Windows CI checks the generated reference against its actual Windows binary; CI does not establish a native UI pass.
- GitHub's renderer returned valid GFM for every page; pixel-level GitHub/browser layout inspection was not performed.
- The website's rendering, release-tag ingestion and `/docs` deployment belong to its repository. README website links remain deferred until that route is confirmed live.

## Regression suite

No live case was promoted in the 0.16.3 sweep. The change is documentation; CI's guide CLI check on both platforms keeps the generated reference current, and the walkthrough gap above is not a maintained case. The [suite's ledger](../regression/README.md#not-carried-over) records this exclusion.

CI results belong to the PR checks. This record reports the local checks above and does not substitute for the required macOS and Windows CI jobs.
