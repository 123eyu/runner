# #1023 — CI speed validation

Two complete warm runs at f912220e passed all four jobs and all six macOS/Windows stress iterations without retries. Windows job time fell 22–28% and macOS job time fell 12–20%; the measured totals still include temporary validation repetitions that are removed from the final workflow. The shipped app is unchanged. Windows CI's generic interactive echo/reattach fixture now uses cmd.exe; it no longer runs that matrix in PowerShell, an operator-accepted coverage tradeoff described below.

| Platform | Baseline job | Warm 1 job | Warm 2 job | Baseline execution | Warm 1 execution | Warm 2 execution |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Windows | 17m44s | 12m46s | 13m47s | 592.030 s | 428.433 s | 442.389 s |
| macOS | 11m04s | 9m42s | 8m51s | 251.757 s | 244.816 s | 218.764 s |
| Linux x86_64 | 7m41s | 6m19s | 7m13s | 189.191 s | 160.930 s | 182.526 s |
| Linux aarch64 | 7m03s | 6m31s | 6m41s | 160.788 s | 158.284 s | 159.958 s |

## Scope and checks

This change measures fixture setup, reduces repeated fixture work, and schedules independent CLI golden cases with every existing assertion retained. The shipped app must behave the same. No live regression case applies to CI and test-only changes; `make verify`, complete nextest runs on all four CI jobs, repeated affected tests, the golden inventory, and all existing workflow gates are required.

Before local checks: fixture probes distinguish GPUI context/window construction from SQLite migration/durability costs; targeted GPUI and CLI tests retain their existing rendering/command assertions; golden inventory/update checks preserve the complete ordered output corpus; repeated nextest runs check fixture, file, process and listener isolation; `make verify` checks the whole workspace with plain Cargo and Clippy, with `CLAUDE_CONFIG_DIR` unset. No cache policy was changed.

## Baseline

[Run 40681403222](https://github.com/yicheng47/runner/actions/runs/40681403222), 2026-10-28, restored a full Rust cache match on every platform. Test execution is nextest's own summary; the Test step also includes compilation. Totals use the first through last job-step timestamps.

| Platform | Clippy | Updater Clippy | Test compilation | Test execution | Test step | App resource build | CLI reference | Installer / packaging | Job | Passed / skipped |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| macOS | 85 s | 46 s | 172 s | 251.757 s | 427 s | — | 39 s | — | 11m04s | 2,982 / 6 |
| Windows | 82 s | — | 183 s | 592.030 s | 778 s | 55 s | 49 s | 29 s | 17m44s | 2,863 / 4 |
| Linux x86_64 | 47 s | — | 97 s | 189.191 s | 288 s | — | 28 s | 1 s | 7m41s | 2,994 / 7 |
| Linux aarch64 | 37 s | — | 99 s | 160.788 s | 261 s | — | 30 s | 1 s | 7m03s | 2,994 / 7 |

## Measured cause and fixture change

GPUI 0.3.7 `TestAppContext::single()` already uses `NoopTextSystem` and fake windows on every platform; native font enumeration is absent from these fixtures. Many window tests instead create and migrate a file-backed SQLite database for each scenario. The slowest Windows GPUI tests loop over many scenarios: `confirm_binding_precedes_every_control_and_preserves_open_menu_values` takes 48.498 s and `sidebar_footer_fits_three_weekly_values_and_avatar_in_one_row` takes 36.617 s.

Temporary diagnostic tests compare fake GPUI construction with fresh SQLite pools using FULL (the default) and NORMAL synchronous modes. Local macOS CI-profile measurements: first context + window 3.36 ms, subsequent pairs 0.06–0.07 ms; fresh seeded FULL pools 11.70–22.10 ms including destruction; NORMAL pools 9.19–10.14 ms. The Windows comparison below was collected before selecting the fixture change.

The CLI help golden takes 97.950 s on Windows and 7.933 s on macOS. It runs all help commands serially inside the two-slot `real-daemon` group, despite never starting a daemon. Real daemon tests retain their two-slot cap unless resource measurements justify changing it.

## Golden scheduling and shared state

The three long golden tests are replaced by 75 help tests, seven inside-mission tests (each retains text and JSON), and seven offline tests: 89 instead of three, a net increase of 86. The manifest still compares all context/argv headings in exact order, and `help_goldens_cover_every_visible_command` still recursively checks the built CLI's visible commands against the help table. The committed golden corpus is unchanged. The helper table now names every help test explicitly so nextest can schedule it independently.

Help cases keep isolated HOME, USERPROFILE, APPDATA, XDG_DATA_HOME, PATH and unique endpoints, but use the already-built CLI without seeding an unused database or installing binary aliases. Every other case retains the seeded fixture and aliases. Inside-mission and offline cases have independent temporary roots; fixtures never use real accounts or real agents. Unix sockets live beneath those roots and Windows pipe names contain the unique root name. Help commands start one CLI child, no daemon, shell or listener. The two-thread `real-daemon` group remains unchanged for all golden, daemon-process and hook-process tests; there is no concurrency-cap increase.

All normal tests read the golden file; the manifest/update test remains its only writer when UPDATE_RUNNER_GOLDENS is set. The process-local lock remains for plain Cargo. The existing follower, restart and hook fixtures retain their original lifecycle/timing assertions.

Three local nextest stress iterations passed all 89 split cases with the original fixture, in 231.573 s total while the development-profile build also ran. After using the lightweight help fixture, all three iterations passed again in 57.378 s total. Individual help cases then took roughly 0.02–0.03 s, compared with the original monolithic local help test's 74.164 s; the slow inside-mission and offline variants still used the original seeded fixture in both measurements. Local targeted GPUI baseline: confirm bindings 0.555 s; sidebar footer matrix 0.702 s.

## Workflow and coverage audit

Nextest's workspace integration-test build already produces target/ci/runner-agent-cli and target/ci/Runner (with .exe on Windows). Resource and generated-reference checks use those exact binaries, preserving the actual checks and avoiding a second Cargo build with a different development-dependency feature graph. On Windows, the manifest/icon check now inspects the Runner.exe built by nextest rather than a separate cargo build; the same runner-app build.rs embeds its manifest and icon in that ordinary executable. The workflow key input, cache scope/saving policy, CI profile, toolchain and required check names remain unchanged. However, [rust-cache v2 hashes every workspace member Cargo.toml](https://github.com/Swatinem/rust-cache/blob/v2/src/config.ts#L172-L210), so adding the test-support feature/dev-dependencies changes its dependency-hash suffix. Exact-key hits and successful prefix restores must be reported separately from each run's actual logs; both restore dependencies and qualify as warm. No cache configuration was changed, so no cold-build measurement was selected.

No tests are removed as redundant. Snapshot round trips, immediate restore state, continuation at event boundaries and seeded random offsets, resize/alternate-screen transitions, and parser state cover distinct failure modes and remain intact. All recording fixtures, cell assertions, offsets, random samples and real timing requirements are retained. No live regression case is selected because this diff changes only CI and test fixtures; the reviewer confirmed no live checks are needed.

[Diagnostic run 40691069464](https://github.com/yicheng47/runner/actions/runs/40691069464) at `043259c6` passed all four jobs with the original fixtures plus two temporary probe tests. Every cache restore was a full match. Its normal test execution was 249.826 s on macOS and 714.599 s on Windows, illustrating hosted-run variability against the original 251.757 s / 592.030 s baseline. The probes were removed after measurement. They built the identical eight-connection, fully migrated, seeded pool with only the connection's synchronous setting changed, for three fresh-file trials per mode. The GPUI probe constructed/dropped an empty fake window three times. The explicit probe step also incurred a separate subset-build feature-graph rebuild (118 s macOS, 99 s Windows); that diagnostic overhead is removed from the final workflow.

| Probe | macOS hosted | Windows hosted |
| --- | --- | --- |
| First fake context + window + drop | 3.030 ms | 1.654 ms |
| Subsequent fake context + window + drop | 0.043–0.104 ms | 0.135–0.209 ms |
| FULL SQLite setup, three trials | 39.386 / 33.215 / 22.379 ms | 283.358 / 362.116 / 512.731 ms |
| NORMAL SQLite setup, three trials | 11.995 / 12.290 / 20.794 ms | 49.040 / 45.003 / 58.605 ms |
| FULL SQLite setup + destruction | 24.417–42.265 ms | 303.465–528.088 ms |
| NORMAL SQLite setup + destruction | 13.725–22.337 ms | 96.734–124.112 ms |

Repeated SQLite synchronization, rather than native GPUI/font/window setup, accounts for the large Windows fixture cost. `db::open_fixture_pool`, available only under test configuration or the test-support feature, uses synchronous=NORMAL for disposable UI and golden fixtures. It still uses the same file-backed WAL, migrations, default seed, foreign keys, five-second SQLite busy timeout, eight connections and 30-second pool checkout timeout. [SQLite documents](https://www.sqlite.org/pragma.html#pragma_synchronous) that NORMAL in WAL mode retains transactional consistency and application-crash persistence while omitting the extra per-commit power-loss sync. These fixtures do not test power loss. The shipped open_pool path, existing runner-daemon database tests, bootstrap tests and real daemon connections retain the original production configuration.

The new `fixture_pool_preserves_file_backing_schema_and_connection_guards` test checks settings on all eight checked-out connections, the latest migration and seeded crew, and data surviving a clean close/reopen through production open_pool with FULL synchronization. The measured test-count change is +87: +86 from the golden split and one fixture-pool regression. The two diagnostic probes are not retained. No ignored set changes.

The first local `env -u CLAUDE_CONFIG_DIR make verify` passed the complete workspace Cargo tests, check, both Clippy passes and formatting before the measured fixture change. A final pass after that change also passed. The reviewer independently cleared the CLI/workflow portion, verified both generated-reference locales and the manifest/visible-help inventories, and confirmed nextest produces the ordinary app/CLI executables needed by the retained gates. Final full-branch review and the operator's own diff review still gate marking the draft ready.

After the measured fixture change, all 92 selected tests (89 split goldens, both window matrices and the fixture-pool regression) passed three local nextest stress iterations without retries, in 44.821 s total. The window matrices retained all dimensions and assertions; fixture-pool checks passed on all eight connections. The subsequent full-workspace make verify passed, as recorded below.

The final local `env -u CLAUDE_CONFIG_DIR CARGO_BUILD_JOBS=12 make verify` passed check, all Cargo tests, workspace Clippy, updater Clippy and formatting. Raw Cargo totals are 3,070 passes and six ignored, including the pre-existing nested Unix daemon subprocess test; the nextest inventory is 3,069 runnable tests and six ignored across the same 25 binaries. A name-set comparison against all 2,982 macOS baseline PASS entries confirms that only the three intended goldens disappear, replaced by their 89 cases and the new database regression. No runnable test is filtered out.

## First post-change run and Windows fixture failure

[Run 40694277623](https://github.com/yicheng47/runner/actions/runs/40694277623), `a57acd66`, restored the existing dependency cache by prefix on every platform (all reported full match: false). macOS passed 3,069 tests in 265.554 s, then all 91 selected cases in three stress iterations (14.646 s execution; 18 s step). Linux x86_64 passed 3,081 in 183.217 s and aarch64 passed 3,081 in 161.951 s. Their job totals were 11m13s, 7m19s and 7m23s respectively.

Windows completed 2,950 tests in 493.286 s: 2,949 passed and one failed, with four existing skipped tests. The failure was `runner-cli::daemon_process shell_echo_two_clients_and_identical_reattach_snapshot`: both subscriptions timed out waiting for SOCKET_ECHO_645 at daemon_process.rs:472. It is distinct from the brief's known companion-frame flake. The resource, repeat and CLI-reference steps did not run after this failure, so its 14m30s job is not a passing end-to-end measurement and is not used as one. No job was blindly rerun.

The generic daemon fixture pinned /bin/sh on Unix but left Windows SHELL to the host, where the resolver prefers pwsh over powershell and ComSpec. Its output wait began immediately after PTY creation; the failed run provides no shell-ready or missing-output transcript, so it cannot establish whether shell startup, console input readiness or output rendering caused the timeout. The fixture now explicitly uses ComSpec on Windows, eliminating the unrelated host-dependent PowerShell/profile startup from this generic IPC/snapshot test. Both ten-second deadlines, concurrent subscriptions, byte equality, snapshot sequence/bytes/size/parser state, and process cleanup assertions stay unchanged. The production default-shell resolver and `shell_command_prefers_pwsh_then_powershell_then_comspec`, native command reporter tests and Windows process tests retain their PowerShell resolution, reporting and process checks. They do not provide an equivalent interactive PowerShell echo/reattach matrix; this generic matrix now uses cmd.exe on Windows. The failed test is added to the temporary hosted stress selection for repeated validation of the fixture fix. This change requires renewed review and fresh complete hosted measurements.

The fixture-isolation delta received a clean code review. The unchanged macOS shell matrix passed three local stress iterations in 3.916 s; this cannot validate the cfg(windows) environment pin. Fresh Windows CI and its repeated matrix are required. The operator's initial code review covered a57acd66; they subsequently accepted the Windows fixture delta with the explicit conditions recorded below.

## Cache evidence

The initial baseline and original-fixture diagnostic run were exact-key hits. The first manifest candidate instead requested suffix 74bfff59 on macOS/Linux and 116b1957 on Windows, then restored the old suffix 65475119 (macOS/Linux) or a5a6dd51 (Windows) through the unchanged restore prefix. This is a warm dependency restore, not an exact-key hit. The workflow's explicit key input and cache policy are unchanged; the suffix difference comes from rust-cache's normal hashing of the new member-manifest feature declarations. No cache strategy is introduced or changed, and no cold-build optimization is claimed. Both passing measurements below independently confirm the actual restore; a successful prefix restore is recorded as warm without implying that every changed feature artifact was already compiled.

The operator accepted the cmd.exe fixture isolation on 2026-10-28 with explicit coverage disclosure. The operator reported scanning the last 60 CI runs: this test failed only on this branch in run 40694277623, never on main or other PRs. Treat this as likely caused by this branch's heavier parallel load, not as an existing flake. The exact failure mechanism remains unproven. Windows CI no longer runs an interactive PowerShell echo/reattach session; its generic IPC fixture uses cmd.exe. The operator requires that tradeoff to appear plainly in the PR description and the Windows stress step to pass this test before ready status.

## Passing warm measurements

Warm run 1 is [run 40697940266, attempt 1](https://github.com/yicheng47/runner/actions/runs/40697940266/attempts/1), commit f912220e. All four jobs passed. Each restored the previous dependency cache through a prefix, with full match: false. Job / normal Test step / normal test execution: Windows 766 / 563 / 428.433 s; macOS 582 / 394 / 244.816 s; Linux x86_64 379 / 244 / 160.930 s; Linux aarch64 391 / 264 / 158.284 s. The Windows resource gate passed against the nextest-built Runner.exe. Every original installer, packaging, updater-Clippy, formatting, reference and lockfile gate applicable to its platform passed.

Both macOS and Windows additionally passed all 92 selected tests in all three stress iterations. Their normal test execution above excludes this validation-only repetition; job totals include it: 18 s on macOS (14.865 s execution) and 39 s on Windows (37.079 s execution). Windows shell_echo_two_clients_and_identical_reattach_snapshot passed in the normal suite (1.125 s) and all three stress iterations (1.133 / 0.945 / 1.179 s), satisfying the operator's stress condition. Warm run 2 intentionally reran the completely passing workflow at the same f912220e commit without a new push, solely for the required second measurement; no tests or assertions changed between measurements. After both complete passing measurements, the temporary stress steps were removed from the final workflow.

Warm run 2 is [run 40697940266, attempt 2](https://github.com/yicheng47/runner/actions/runs/40697940266/attempts/2), the identical f912220e commit. All four jobs and every existing applicable gate passed again, with the same counts: Windows 2,950 passed / four skipped, macOS 3,069 / six, and each Linux architecture 3,081 / seven. All four cache restores were prefix hits with full match: false, restoring the same suffixes as warm run 1. Windows passed all 92 selected cases in three stress iterations (40.704 s execution, 43 s step); macOS did the same (15.118 s execution, 18 s step). This includes the Windows shell echo/reattach matrix in every iteration. No passing result depends on retries.

The macOS job in warm run 2 waited about five minutes for a hosted runner before its active 8m51s job began. Tables report active job duration from GitHub's started/completed job timestamps, excluding that queue delay. Normal Test execution excludes compilation and the separate validation-only stress step. The full per-step tables below include that step explicitly; it is absent from the final steady-state workflow, so its overhead is not hidden in a claim about steady-state performance. No final-head timing is inferred by subtracting it.

The Windows runner-app sum of individual test durations fell from 935.2 s to 438.8 / 501.8 s, with unchanged 934 runnable app tests; p90 fell from about 2.45 s to 1.264 / 1.501 s. These are aggregates of individual test durations, not additional job wall time. The difference from the empty-window probe supports removing repeated database synchronization rather than changing native text/font setup. Windows remains the slowest job, and hosted timing variability is visible in both warm measurements.

## Reproduction and final workflow

The normal CI command remains unchanged:

```sh
cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci --timings
```

The temporary stress command executes three iterations per platform in each passing measurement and is retained here for reproduction rather than in the steady-state workflow:

```sh
cargo nextest run --locked --workspace --cargo-profile ci --no-fail-fast --stress-count 3 -E 'test(built_cli_goldens_help) | test(built_cli_goldens_inside_mission) | test(built_cli_goldens_not_running) | test(confirm_binding_precedes_every_control) | test(sidebar_footer_fits_three_weekly) | test(shell_echo_two_clients_and_identical_reattach_snapshot)'
```

Final local gates were `env -u CLAUDE_CONFIG_DIR CARGO_BUILD_JOBS=12 make verify`, the 92-case/three-iteration local stress selection including the new pool regression, the separate shell-matrix three-iteration run, `cargo nextest list --locked --workspace --cargo-profile ci --message-format json`, and `python3 script/generate-cli-reference.py --runner target/ci/runner-agent-cli --check`. The new Windows-only child environment is compiled out locally and is validated by the two passing hosted Windows runs. The committed golden file is byte-unchanged. No live app checks were needed, as confirmed by the reviewer.

The final workflow removes only the temporary repetitions after validation. It keeps the original normal test command, all formatting/Clippy/updater/installer/resource/reference/lockfile/Linux-packaging gates, the same four platform jobs and required names, cache configuration, and the two-thread real-daemon cap. The remaining final delta is workflow cleanup and this evidence record; final-head CI, the reviewer's delivery sign-off and the operator's final review gate still apply.

## Complete per-step measurements

Values are seconds unless marked. Warm columns are the two complete passing attempts above; the failed a57acd66 attempt and the temporary diagnostic run are recorded separately and are not counted as passing measurements. Removed redundant builds are shown as an em dash; their resource/reference gates remain present.

### Rust / Windows

| Step / metric (seconds unless marked) | Baseline | Warm 1 | Warm 2 |
| --- | ---: | ---: | ---: |
| Set up job | 2 | 2 | 2 |
| Check out repository | 8 | 7 | 11 |
| Install Rust toolchain | 16 | 16 | 17 |
| Install nextest | 1 | 1 | 1 |
| Remove other Rust toolchains | 1 | 1 | 1 |
| Hash build settings | 0 | 0 | 0 |
| Cache Rust build | 32 | 26 | 26 |
| Verify Windows installer behavior | 29 | 36 | 31 |
| Clippy | 82 | 67 | 76 |
| Test | 778 | 563 | 608 |
| Test compilation | 183 | 133 | 164 |
| Test execution | 592.030 | 428.433 | 442.389 |
| Repeat split goldens and window fixtures | — | 39 | 43 |
| Build Runner for resource verification | 55 | — | — |
| Verify Runner manifest and icon | 1 | 1 | 1 |
| Check generated CLI reference | 49 | 2 | 3 |
| Verify lockfile is committed | 1 | 0 | 0 |
| Upload build timings | 2 | 2 | 2 |
| Job total | 1064 | 766 | 827 |
| Passed / skipped | 2863 / 4 | 2950 / 4 | 2950 / 4 |
| Cache restore | exact | prefix | prefix |
| Restored suffix | a5a6dd51 | a5a6dd51 | a5a6dd51 |

### Rust / Linux (x86_64)

| Step / metric (seconds unless marked) | Baseline | Warm 1 | Warm 2 |
| --- | ---: | ---: | ---: |
| Set up job | 1 | 1 | 1 |
| Initialize containers | 2 | 2 | 4 |
| Bootstrap checkout prerequisites | 8 | 11 | 12 |
| Check out repository | 3 | 3 | 3 |
| Install Linux dependencies | 18 | 20 | 16 |
| Install Node for embedded pi extension tests | 5 | 4 | 6 |
| Install Rust toolchain | 9 | 9 | 9 |
| Install nextest | 1 | 1 | 1 |
| Cache Rust build | 37 | 32 | 43 |
| Check formatting | 6 | 4 | 6 |
| Clippy | 47 | 40 | 46 |
| Test | 288 | 244 | 276 |
| Test compilation | 97 | 81 | 92 |
| Test execution | 189.191 | 160.930 | 182.526 |
| Repeat split goldens and window fixtures | — | — | — |
| Check generated CLI reference | 28 | 0 | 1 |
| Test Linux installer ownership | 1 | 1 | 1 |
| Check Linux desktop entry | 0 | 0 | 0 |
| Verify lockfile is committed | 0 | 1 | 0 |
| Upload build timings | 2 | 1 | 2 |
| Stop containers | 2 | 2 | 1 |
| Job total | 461 | 379 | 433 |
| Passed / skipped | 2994 / 7 | 3081 / 7 | 3081 / 7 |
| Cache restore | exact | prefix | prefix |
| Restored suffix | 65475119 | 65475119 | 65475119 |

### Rust / Linux (aarch64)

| Step / metric (seconds unless marked) | Baseline | Warm 1 | Warm 2 |
| --- | ---: | ---: | ---: |
| Set up job | 1 | 1 | 2 |
| Initialize containers | 5 | 4 | 4 |
| Bootstrap checkout prerequisites | 10 | 11 | 11 |
| Check out repository | 2 | 2 | 2 |
| Install Linux dependencies | 19 | 18 | 20 |
| Install Node for embedded pi extension tests | 4 | 4 | 5 |
| Install Rust toolchain | 7 | 7 | 7 |
| Install nextest | 1 | 1 | 0 |
| Cache Rust build | 31 | 28 | 30 |
| Check formatting | 6 | 6 | 5 |
| Clippy | 37 | 38 | 39 |
| Test | 261 | 264 | 269 |
| Test compilation | 99 | 105 | 107 |
| Test execution | 160.788 | 158.284 | 159.958 |
| Repeat split goldens and window fixtures | — | — | — |
| Check generated CLI reference | 30 | 1 | 1 |
| Test Linux installer ownership | 1 | 1 | 0 |
| Check Linux desktop entry | 0 | 0 | 0 |
| Verify lockfile is committed | 0 | 0 | 1 |
| Upload build timings | 2 | 1 | 1 |
| Stop containers | 3 | 1 | 1 |
| Job total | 423 | 391 | 401 |
| Passed / skipped | 2994 / 7 | 3081 / 7 | 3081 / 7 |
| Cache restore | exact | prefix | prefix |
| Restored suffix | 65475119 | 65475119 | 65475119 |

### Rust / macOS

| Step / metric (seconds unless marked) | Baseline | Warm 1 | Warm 2 |
| --- | ---: | ---: | ---: |
| Set up job | 1 | 1 | 2 |
| Check out repository | 4 | 4 | 3 |
| Install Rust toolchain | 12 | 11 | 12 |
| Install nextest | 1 | 1 | 1 |
| Remove other Rust toolchains | 0 | 0 | 0 |
| Hash build settings | 0 | 0 | 0 |
| Cache Rust build | 22 | 23 | 22 |
| Check formatting | 9 | 5 | 5 |
| Install Metal toolchain | 10 | 9 | 7 |
| Clippy | 85 | 65 | 60 |
| Clippy with updater | 46 | 43 | 38 |
| Test | 427 | 394 | 354 |
| Test compilation | 172 | 146 | 133 |
| Test execution | 251.757 | 244.816 | 218.764 |
| Repeat split goldens and window fixtures | — | 18 | 18 |
| Check generated CLI reference | 39 | 1 | 2 |
| Verify lockfile is committed | 0 | 0 | 0 |
| Upload build timings | 2 | 2 | 2 |
| Job total | 664 | 582 | 531 |
| Passed / skipped | 2982 / 6 | 3069 / 6 | 3069 / 6 |
| Cache restore | exact | prefix | prefix |
| Restored suffix | 65475119 | 65475119 | 65475119 |
