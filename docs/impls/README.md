# Implementation plans

An impl plan is tactical: the spec says *what*, the plan says how a change lands, what it touches and how it is verified. Architecture truth lives in [`../arch/`](../arch/); fold a decision that outlives the work into it before archiving the plan.

A plan is written on its feature's branch and lands in the pull request with the code. It stays here while the feature's remaining work is in flight and moves to [`archive/`](./archive/) when it ships, keeping its filename so links stay stable. Since 2026-09-01 plans are named `{number}-{slug}.md` after their feature spec; the archived `0001`–`0045` sequence is closed. Mission briefs live in [`briefs/`](./briefs/), named `{number}-{slug}.md`, or `{number}-m{n}-{slug}.md` when a feature takes several missions. Shipped briefs are pruned except the references listed in [`briefs/README.md`](./briefs/README.md); the rest are in git history.

## Active

- [`562-mission-spawn/`](./562-mission-spawn/README.md) — missions as containers ([#562](https://github.com/yicheng47/runner/issues/562)): Phase 1 shipped in [#868](https://github.com/yicheng47/runner/pull/868), session independence in [#874](https://github.com/yicheng47/runner/pull/874), Phase 2's adding and removing roles in [#889](https://github.com/yicheng47/runner/pull/889) (0.16.0) and Phase 3's empty start in [#906](https://github.com/yicheng47/runner/pull/906) (0.16.2); the lead's own verbs and outside seats remain.

## Archive

- [`733-gpui-pre.md`](./archive/733-gpui-pre.md) — move GPUI from the yanked `gpui-ce` 0.3.3 to `gpui-pre =0.3.7` ([#767](https://github.com/yicheng47/runner/pull/767), shipped in 0.12.6 on 2026-10-01).
- [`878-session-launch-definition.md`](./archive/878-session-launch-definition.md) — record each session's launch inputs at spawn so Resume and fork ignore later role edits ([#881](https://github.com/yicheng47/runner/pull/881), shipped in 0.16.0 on 2026-10-13).
- [`701-desktop-notifications.md`](./archive/701-desktop-notifications.md) — system notifications when an agent needs you, and the Settings → Notifications pane ([#891](https://github.com/yicheng47/runner/pull/891), shipped in 0.16.1 on 2026-10-14).

Shipped plans are in [`archive/`](./archive/) in number order, and each names its PR. Multi-mission programs keep a folder with a condensed README:

- [`645-runnerd/`](./archive/645-runnerd/README.md) — `runnerd`, the daemon that owns sessions and state so they outlive the app ([#645](https://github.com/yicheng47/runner/issues/645), shipped 2026-10-06 in 0.13.0 through four missions, [#805](https://github.com/yicheng47/runner/pull/805) to [#811](https://github.com/yicheng47/runner/pull/811)); the CLI followed onto its client protocol in [#822](https://github.com/yicheng47/runner/pull/822) (#821), and remote machines continue as [#808](https://github.com/yicheng47/runner/issues/808).
- [`648-runner-cli/`](./archive/648-runner-cli/README.md) — the general `runner` CLI, its agent skill and the MCP removal ([#648](https://github.com/yicheng47/runner/issues/648), shipped 2026-09-20 in 0.11.0 through five missions, [#650](https://github.com/yicheng47/runner/pull/650) to [#667](https://github.com/yicheng47/runner/pull/667)); the Windows checks continue as [#668](https://github.com/yicheng47/runner/issues/668).
- [`539-pi-runtime/`](./archive/539-pi-runtime/README.md) — pi as a runtime with hook status ([#539](https://github.com/yicheng47/runner/issues/539), shipped 2026-09-20 in 0.11.0 in [#646](https://github.com/yicheng47/runner/pull/646) and [#649](https://github.com/yicheng47/runner/pull/649)); the fixture and the JASONPC pass continue as [#668](https://github.com/yicheng47/runner/issues/668).
- [`347-hook-status/`](./archive/347-hook-status/README.md) — hook-based agent status on macOS ([#347](https://github.com/yicheng47/runner/issues/347), closed 2026-09-16); Windows followed as [610](./archive/610-windows-hook-status.md), shipped in 0.10.0.
- [`604-role-rename/`](./archive/604-role-rename/README.md) — the runner entity became role ([#604](https://github.com/yicheng47/runner/issues/604), shipped 2026-09-16 in [#618](https://github.com/yicheng47/runner/pull/618)).
- [`local-skills/`](./archive/local-skills/README.md) — the Skills pane program ([#73](https://github.com/yicheng47/runner/issues/73), closed 2026-09-13; per-role picks continue as [#577](https://github.com/yicheng47/runner/issues/577)).
- [`gpui-rewrite/`](./archive/gpui-rewrite/README.md) — the GPUI rewrite, shipped as `v0.6.0` on 2026-08-23; its dated log, surface inventory and mission briefs were pruned on 2026-09-18 and live in git history.
- [`windows-nightly/`](./archive/windows-nightly/README.md) — the Windows port ([#437](https://github.com/yicheng47/runner/issues/437)), shipped in 0.8.0; current build instructions are in [`../arch/windows.md`](../arch/windows.md).
