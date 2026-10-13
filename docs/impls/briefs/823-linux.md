# 823 — Linux x86_64 release: runnerd, the CLI and the app

Deliver [#823](https://github.com/yicheng47/runner/issues/823). Jason requested this mission on 2026-10-09 for 0.14 (renumbered 0.16 on 2026-10-10): a Linux x86_64 build of the whole product, the GPUI app, `runnerd` and the `runner` CLI, built and released from CI beside macOS and Windows. His own use is `runnerd` on a Linux machine he reaches over ssh; the issue's requester runs WSL2. Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/feat-823-linux`, on the branch `feat/823-linux`, brought up to current `origin/main` before the mission starts. This brief landed on main ahead of the mission (Jason, 2026-10-09), so it is not part of the mission's commit. The root checkout stays on main. Do not create another branch or worktree, and do not share a Cargo target directory.

This machine is a Mac with no Linux host, container runtime or VM. The `Rust / Linux` CI job is the build and test loop for anything Linux-only, and Jason runs the live checks. The headless crates can be cross-checked here: the `x86_64-unknown-linux-gnu` Rust target and `zig` are installed, and `cargo clippy --target x86_64-unknown-linux-gnu -p runner-core -p runner-terminal -p runner-daemon -p runner-cli --all-targets` works with `CC_`, `CXX_`, `AR_` and the linker for that target pointed at wrappers around `zig cc -target x86_64-linux-gnu.2.28` that drop cc-rs's `--target=x86_64-unknown-linux-gnu` argument, which zig rejects. Use a target directory inside this worktree. The app needs Linux system libraries and builds only in CI.

## Decisions already made (Jason, 2026-10-09)

- x86_64 only (`x86_64-unknown-linux-gnu`). No aarch64.
- One mission for all of it: daemon, CLI, app, CI, packaging, release and docs.
- Formats: a `.tar.gz` with an `install.sh` as the universal format (the way Zed ships GPUI on Linux), plus `.deb` and `.rpm` built by nfpm from the same staged files. No AppImage, Flatpak or Snap: AppImage needs FUSE and its runtime environment leaks into the agents Runner spawns, and the Flatpak and Snap sandboxes block spawning arbitrary CLIs.
- No updater in v1. Linux has no update UI at all, rather than a disabled Sparkle path.
- `runnerd` gets a systemd user unit, opt-in.
- Window chrome and shortcuts follow Windows: Runner draws its own title bar and Ctrl is the primary modifier. No Pencil work. If a surface genuinely needs a design decision, stop and ask Jason.
- Build on Ubuntu 22.04 (glibc 2.35), so the binaries run on current distros.

## What already holds (checked 2026-10-09)

- `runner-core`, `runner-terminal`, `runner-daemon` and `runner-cli` pass Clippy with warnings denied and `--all-targets` for `x86_64-unknown-linux-gnu`. Their tests have never run on Linux.
- `runnerd` and `runner` are one binary, `runner-agent-cli`, which picks its mode from argv[0] (`runner-cli/src/main.rs` about line 80).
- Linux paths use `XDG_DATA_HOME` (`runner-core/src/app_paths.rs`), and `process_command_line` already reads `/proc/<pid>/cmdline` (`session/process/unix.rs` about line 268).
- `gpui-pre-linux` (X11 and Wayland) is in `Cargo.lock`, pulled in by `gpui-pre-platform` on Linux. `platform_ui/windows.rs` uses only GPUI, no Win32.

## Read first

- `AGENTS.md`, including Development Commands, Worktrees and Crew Missions.
- Issue #823 and its comments.
- `docs/arch/arch.md` line 88 (Platform target) and §14 (release and nightly channels); `docs/arch/windows.md`, the model for a platform doc; `docs/arch/process-model.md`.
- `crates/runner-app/Cargo.toml`: `gpui` is declared only for the macOS and Windows targets, so the app does not compile on Linux yet.
- `crates/runner-app/src/lib.rs` (`platform_fonts`, `wake`), `platform_ui/mod.rs`, `platform_ui/windows.rs`, the `fonts_*.rs` files, `mac_chrome.rs`, `keymap.rs`, `updater.rs`, and `main.rs` (gates near lines 334, 551, 939, 987, 1206 and 1396–1428).
- `grep -rnE 'cfg\((not\()?(target_os|windows|unix)|cfg!\(' crates/runner-app/src`: about 190 gates in 50 files. There, `cfg(not(windows))` means macOS in practice ("Reveal logs in Finder", ⌘H Hide and Hide Others, Sparkle hints).
- `crates/runner-daemon/src/session/process/unix.rs` (`live_descendants`, about lines 115 and 142) and `session/codex_capture.rs` (`open_rollout_paths_for_pid`, about lines 485 and 506).
- `crates/runner-core/src/daemon_process.rs` (`Launch::spawn`, the mismatch path in `connect_or_spawn_with_restart` about line 173, `startup_lock`), `runner-app/src/bootstrap.rs` (the sidecar hash), `runner-core/src/cli_install.rs` (`locate_source`), `runner-daemon/src/cli_install.rs` (the `~/.local/bin` link and its shadowing check), `runner-cli/src/client.rs` (about line 70: no auto-start under `SSH_CONNECTION`; `NOT_RUNNING_MESSAGE`) and `runner-daemon/src/shell_path.rs`.
- `.github/workflows/ci.yaml`, `release.yml`, `nightly.yml`; `script/bundle-mac`, `script/bundle-windows.ps1`, `script/nightly-release-notes.md`; `.claude/skills/nightly/SKILL.md` and `.claude/skills/release/SKILL.md`; `Makefile`.

## Deliverables

Work in this order, and have the `Rust / Linux` job green on the headless crates before starting on the app.

1. **CI.** A `Rust / Linux` job in `ci.yaml` on `ubuntu-22.04` with the macOS job's steps (format check, Clippy with warnings denied, nextest with the `ci` profile, the lockfile check, timings), plus the apt packages GPUI needs. It also builds the packages (deliverable 5), installs the tarball into a temporary prefix and inspects the `.deb` and `.rpm`, as the Windows job checks its installer. `Rust / macOS` and `Rust / Windows` stay unchanged. Branch protection is Jason's.
2. **Daemon and CLI on Linux.** `live_descendants` from `/proc`, so descendants that outlive a stopped agent are reaped as on macOS. `open_rollout_paths_for_pid` from the `/proc/<pid>/fd` links, so codex capture is pid-assisted as on macOS. Fix whatever the first Linux test run exposes; a test that cannot hold on Linux gets a `cfg` with a one-line reason, never a silent skip.
3. **The systemd user unit.** Ship `runnerd.service` in all three formats (`/usr/lib/systemd/user/` in the packages, `~/.config/systemd/user/` from `install.sh`); nothing enables it on install. Requirements:
   - It runs the same bytes the app would, so the app's hash check accepts it and connects rather than replacing it.
   - The unit and the app never fight. When the app replaces a mismatched daemon after an upgrade, or when another daemon already owns the endpoint, systemd neither restart-loops nor starts a second daemon. Choose the exit status, `Restart=` policy and `startup_lock` handling, and say why in the handoff.
   - A daemon started by systemd gets a minimal environment. Confirm `shell_path` discovery still gives agents the login-shell `PATH`.
   - Over ssh, `runner` still never starts a daemon. On Linux, `NOT_RUNNING_MESSAGE` names `systemctl --user start runnerd` beside opening Runner.
4. **The app on Linux.**
   - `runner-app/Cargo.toml` gets Linux target entries for `gpui-pre` (X11 and Wayland), its platform crate and the test-support dev-dependency, matching how macOS and Windows are declared.
   - Add `platform_ui/linux.rs` and `fonts_linux.rs`, with fontconfig fallbacks that cover CJK (Noto Sans CJK) before `sans-serif` and `monospace`. Share code with `windows.rs` where it is identical instead of copying it.
   - Every macOS- or Windows-only gate in `runner-app` gets a deliberate Linux answer: Windows' shortcut bindings, no Hide or Hide Others, folders opened with `xdg-open` under neutral wording, no update UI, no wake observer.
   - Gaps that stay in v1 are listed in the platform doc, among them clipboard file paths.
5. **Packaging.** `script/bundle-linux` stages a release build: `Runner`, the CLI sidecar as `runner-agent-cli` beside it (where `locate_source` looks), `runnerd.service`, a `.desktop` file and the icons. It produces `Runner-<version>-x86_64.tar.gz` with `install.sh`, which installs under `~/.local`, adds the desktop entry and icon, installs the unit file and has `--uninstall`. It also produces `.deb` and `.rpm` through nfpm from the same tree, with runtime dependencies declared from the CI job's library list. `runner` must work on a headless machine where the app never ran. Decide how the packages put it on `PATH` without creating a second link that the app's CLI install then reports as foreign or shadowing.
6. **Release and nightly.**
   - `release.yml` gets a `build-linux` job on `ubuntu-22.04` that uploads the tarball, `.deb` and `.rpm` to the same draft, under the same gate as the other platforms.
   - `nightly.yml` gets a `linux` platform choice, and the default builds every platform. Linux assets join the `nightly` prerelease under the shared stamp, are pruned to two builds by stamp independently of the others, and never touch the appcast or the Windows feed.
   - Update the `nightly` and `release` skills and the install text in `script/nightly-release-notes.md`.
   - No signing in v1.
7. **Docs.**
   - A new `docs/arch/linux.md` covering local development with its apt packages, validation, packaging, the unit with `systemctl --user enable --now runnerd` and `loginctl enable-linger`, WSL2 (systemd in `/etc/wsl.conf`, WSLg for the app, and whether WSL stopping an idle distro stops `runnerd`, checked against current WSL documentation) and the known gaps.
   - Update `arch.md` (Platform target, the `platform_ui` sentence, §14), `vision.md` line 144, the download sections of `README.md` and `README.zh-CN.md` together, and in `AGENTS.md` both Development Commands and the `platform_ui/{macos,windows}.rs` convention under Engineering Conventions.
   - Change the `Makefile` only where `make run` or `make verify` fails on Linux.
8. **Test record.** Write `docs/tests/823-linux.md`: the live checks Jason runs on WSL2 with WSLg on his Windows PC, and on a native distro if he has one:
   - installing from the `.deb` and from the tarball;
   - the app on Wayland or WSLg;
   - a Claude Code and a Codex chat with hook status;
   - Chinese input through fcitx5 or ibus in a terminal pane;
   - a mission start and a `runner msg` round trip;
   - Quit leaving sessions running;
   - the unit enabled with linger, surviving a reboot or `wsl --shutdown`, and reachable over ssh with `runner session list`;
   - an upgrade with the unit enabled producing one daemon and no restart loop;
   - uninstalling.

Keep the change scoped: no aarch64, no updater, no Linux-only features beyond parity, and no change to macOS or Windows behavior. The downloads page on runnersh.dev lives outside this repo; say in the handoff what it needs.

## Boundaries

Crews never run the dev app. Do not run `runner` or `runner-dev` commands that start, stop or kill sessions or the daemon. Do not dispatch `nightly.yml` or `release.yml`, create tags or releases, or change branch protection; Jason cuts the first Linux nightly. Do not start extra agents, crews or subagents. Gate imports and helpers used only by `cfg(unix)` tests with `#[cfg(unix)]`, and Windows-only ones with `#[cfg(windows)]`; Windows CI has repeatedly broken on ungated imports.

## Review, verification and authorization

The coder owns implementation and checks. The reviewer waits for an explicit Runner handoff, then reviews the full branch diff against #823 with must-fix findings first and file:line pointers. Focus on:

- macOS and Windows behavior unchanged, with no existing arm altered by a reworded `cfg`;
- every app gate having a deliberate Linux answer;
- the unit and the app never running two daemons or restart-looping;
- the package layout and the `runner` link;
- the Linux job running the whole test suite, with no test dropped and each Linux `cfg` justified;
- the nightly keeping each platform's assets separate.

Iterate until the reviewer posts `NO REMAINING MUST-FIX ISSUES`.

Locally, run workspace Clippy with warnings denied (`cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`), macOS updater Clippy (`--features updater`), `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`, `cargo fmt --all --check` and `git diff --check`, plus the Linux cross-check of the headless crates above. Record the exact commands and exit codes.

After a clean review, Jason authorizes the following:

- Squash all work on this branch into one commit on top of current `origin/main`, with a subject that names the change (for example `feat: ship Runner on Linux x86_64`).
- Push `feat/823-linux` and open a PR against main whose body says `Closes #823`.
- If main has moved, rebase; never merge main into the branch.
- Amend review or CI fixes after the push into the same commit and push with `git push --force-with-lease`.
- Drive CI green on macOS, Windows and Linux.

Do not merge, delete the branch or worktree, or cut a nightly or release.

The final Runner handoff covers:

- the PR URL and what changed;
- the commands and exit codes;
- the CI result;
- the reviewer's verdict;
- the systemd decisions;
- what the downloads page needs;
- a pointer to the test record for Jason's live checks.

Then stand by.
