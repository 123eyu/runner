# 562 — missions as containers: program record

Implementation record for [feature 562](../../features/562-mission-spawn.md) ([#562](https://github.com/yicheng47/runner/issues/562), "delegate work to other agents", milestone 0.16). The spec says *what*; this directory says *how, in what order, and what has landed*. This file is the condensed state and the decisions that bind every mission. [plan.md](plan.md) has each phase in detail, and [impl_log.md](impl_log.md) is the dated log. Briefs go in [`../briefs/`](../briefs/) as `562-m{n}-{slug}.md`. The design is `design/specs/562-mission-spawn.pen` on `main`, drawn phase by phase.

## Status (2026-10-13)

| Mission | Scope | Crew | State |
| --- | --- | --- | --- |
| m1 | Phase 1: each mission keeps its roster in `roster.json` (fixes #863) | — | Merged 2026-10-11 as [#868](https://github.com/yicheng47/runner/pull/868), shipped in 0.15.2 |
| m2 | Each mission session independent of the others (Phase 2 decision 9, split out) | codex trio | Merged 2026-10-12 as [#874](https://github.com/yicheng47/runner/pull/874); QA passed five macOS checks, Windows unverified |
| — | [#878](https://github.com/yicheng47/runner/issues/878): a session records its whole launch definition at spawn, so role edits never change it | codex trio | Merged 2026-10-13 as [#881](https://github.com/yicheng47/runner/pull/881) after live QA on macOS; it also fixed the feed's raw card for a stopped mission. Windows unverified |
| m3 | Phase 2: add and remove roles while a mission runs, CLI and app | codex trio | Planned and briefed on `feat/562-add-remove-roles`, rebased onto #881; waiting for Jason's review of the plan and his go |
| — | Phase 3: start a mission from roles, without a crew | — | Interface under discussion |

Related issue: [#873](https://github.com/yicheng47/runner/issues/873) makes the mission lead optional and may answer Phase 3's first question.

## Decisions that bind

- **A mission owns its roster** (Jason, 2026-10-11). The database holds the definitions that are listed and searched (roles, crews, slots); a mission's roster is its own state, in `roster.json` beside its event log, and a crew edit never changes a mission that has started.
- **Each phase is designed first and ships its GUI with its CLI** (Jason, 2026-10-12). Phase 2 and Phase 3 each draw their frames on `main`, stop for sign-off, and land daemon, CLI and app work in one pull request.
- **Roles and crews are templates; each layer copies at creation** (Jason, 2026-10-12). A mission copies its crew into its roster at start, and a session records its whole launch definition at spawn (#878). Resume replays a session exactly; Restart takes up role edits. The spec's "What is copied, and when" has the rules.
- **Each mission session is independent** (Jason, 2026-10-12). One session's exit or failed spawn never stops another; shipped in m2.
- **Nothing assumes a mission has a lead** (2026-10-12). New code reads the lead from a slot's flag, so #873 stays a local change (plan, Phase 2 decision 14).
- **Missions run on the codex trio** (Jason, 2026-10-12). `qa` runs the mission regression checks in the development app on macOS; Windows is Jason's.
- **The coordinator can sit outside the mission.** A chat that starts a mission drives it through the `runner` CLI; crews stay as templates for repeatable shapes.

## Open

- Jason's go for m3. Its [brief](../briefs/562-m3-add-remove-roles.md) relies on the standing pre-approval of Runner's Codex trust entry for `qa`'s own `/tmp` folders (`docs/tests/full-smoke-test.md`, Agent configuration isolation). Before launch, the six Phase 2 frames are exported as PNGs into the worktree's `target/design-ref/`, under the names the brief lists.
- Phase 3's interface: the spec's questions, and #873.
- m1's light Windows check, at the next update of Jason's PC with existing missions ([record](../../tests/archive/562-mission-roster.md)).
- The release that ships #881 and m3 gates on a Windows PC pass: back up the database, upgrade over 0.15.3 with real data, the suite's smoke cases, and the Windows checks deferred from m1, m2, #881 and m3.
