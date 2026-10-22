# Runtime adapters: a first code walkthrough

Checked against Runner `main` on 2026-10-22. The adapter boundaries came from [#777](../features/archive/777-runtime-adapter.md); daemon ownership, local hook IPC and the shared state reducer followed in [#645](../features/archive/645-session-host.md), [#797](../features/archive/797-hook-status-ipc.md) and [#791](../features/archive/791-session-state.md). The walkthrough below follows the current paths rather than the original file-based hook transport.

The main change is moving each agent's integration code into its own module. Previously, Codex behavior, Claude behavior, and so on were scattered across launch code, status code, settings, and other files. The refactor gives those differences a common interface: `RuntimeAdapter`.

## What “runtime” means here

Several names contain “runtime”, but they describe different responsibilities:

| Name | Responsibility | Code |
| --- | --- | --- |
| `Runtime` | Identifies the agent: Codex, Claude Code, pi, etc.; also includes Shell. | [`runner-core/src/runtime.rs`](../../crates/runner-core/src/runtime.rs) |
| `RuntimeAdapter` | Knows how Runner integrates with that particular agent CLI. | [`runtimes/mod.rs`](../../crates/runner-daemon/src/runtimes/mod.rs) |
| `SessionManager` | Coordinates a session: creation, persistence, spawning, resuming, and updates. | [`session/manager/mod.rs`](../../crates/runner-daemon/src/session/manager/mod.rs) |
| `SessionRuntime` / `PtyRuntime` | Runs the actual process in a pseudo-terminal, carries input/output, resizes it, and tracks process exit. | [`session/runtime.rs`](../../crates/runner-daemon/src/session/runtime.rs), [`session/pty_runtime.rs`](../../crates/runner-daemon/src/session/pty_runtime.rs) |

The important boundary is agent knowledge versus process management. A Codex adapter knows Codex's flags and conversation format. The PTY layer provides the terminal connection through which Codex runs. A pseudo-terminal (PTY) makes the child process behave as though it is connected to an interactive terminal.

## A simplified launch

```mermaid
flowchart TD
    A[Start a chat or mission slot] --> B[SessionManager]
    B -->|Ask for agent-specific behavior| C[RuntimeAdapter]
    C -->|Arguments, environment, resume plan, hooks| B
    B -->|Prepared SpawnSpec| D[PtyRuntime]
    D --> E[Agent CLI process]
    E -->|Terminal output and status evidence| D
    D -->|Output and observations| B
    B --> F[UI updates and persistence]
```

This is a responsibility map, not a complete thread or IO diagram. In particular, status watchers also read hook feeds and the agent's own records; status evidence does not all travel through the PTY byte stream.

`SpawnSpec` is the prepared launch input: command, arguments, environment, working directory, terminal size, and metadata needed by the process layer. `SessionManager` gathers these inputs; `PtyRuntime` does not read the database to discover how to launch the session.

## Before and after the refactor

Before the refactor, shared code repeatedly selected behavior by agent:

```rust
match runtime {
    Runtime::Codex => /* Codex flags */,
    Runtime::ClaudeCode => /* Claude flags */,
    // ...
}
```

That pattern appeared in many places. Supporting another agent meant finding and updating all of them.

Now shared code obtains an adapter and asks it for the behavior:

```rust
let adapter = crate::runtimes::for_key(&role.runtime);
let plan = adapter.resume_plan(prior_key);
```

The registry in [`runtimes/mod.rs`](../../crates/runner-daemon/src/runtimes/mod.rs) selects the implementation. Each agent has a directory under `runtimes/`, containing its adapter and supporting code. Optional trait methods default to no support or no extra behavior. Shell and unknown runtime keys use `NoAgent`.

For example, the [`Codex` adapter](../../crates/runner-daemon/src/runtimes/codex/mod.rs) specifies how to pass the first prompt, resume a conversation, fork it, assemble launch arguments, install status hooks, and expose catalog capabilities. Shared orchestration combines the adapter's answers into a `SpawnSpec` in `SessionManager::apply_runtime_args`, in [`session/manager/spawn.rs`](../../crates/runner-daemon/src/session/manager/spawn.rs).

The resulting division is: the manager coordinates the launch, the adapter supplies agent-specific rules, and the PTY implementation runs the process. The app keeps UI-specific presentation, such as icons and colors, in [`runtime_ui.rs`](../../crates/runner-app/src/runtime_ui.rs).

## Codex status: entry points and one example

Hook setup lives in the [`Codex` adapter](../../crates/runner-daemon/src/runtimes/codex/mod.rs); event parsing lives in [`codex_status.rs`](../../crates/runner-daemon/src/runtimes/codex/codex_status.rs). Shared transport and admission live outside the adapter so every runtime uses the same session/generation and queue limits.

### Launch setup

`status_hooks()` returns the adapter's `Hooks` implementation. `Hooks::env()` adds a session ID, a new spawn generation and the bundled reporter executable when injection is allowed. The endpoint is supplied by the daemon's runtime. Explicit hook-disable or configuration overrides detected by `inject_codex_hooks` leave that configuration alone.

`codex_status_args()` adds invocation-only configuration through separate `-c` key/value arguments. It enables hooks and bypasses their own hook-trust review; those flags are separate from tool approvals and sandbox permissions. The values are assembled with TOML serialization rather than shell concatenation.

Normal Codex events use `type="mcp_tool"` against an optional `runner_hooks` stdio helper whose command is the bundled executable with `hook serve`. The helper exposes only status reporting; it is not the old Runner workspace MCP API. `SessionEnd` uses a command hook calling `runner hook report` because Codex does not support that MCP handler during teardown. Both helpers connect to the existing daemon and cannot start or replace it.

When [`PtyRuntime::spawn`](../../crates/runner-daemon/src/session/pty_runtime.rs) sees the hook generation, it registers a route for the session, runtime and generation and passes the resulting receiver to `Hooks::start_receiver`. Codex builds a `CodexStatusWatcher::from_receiver`, holding the feed receiver, its parser and an optional transcript tail. Stopping the process retires its route.

### What the reporting helper does

The transport contract is [`runner-core/src/protocol/hook.rs`](../../crates/runner-core/src/protocol/hook.rs): `HookReport` carries version, runtime, Runner session ID, spawn generation, event, provider payload and caller-thread identity where required. The route is configured through `RUNNER_HOOK_ENDPOINT`, `RUNNER_HOOK_SESSION` and `RUNNER_HOOK_GENERATION`.

[`runner-cli/src/hook.rs`](../../crates/runner-cli/src/hook.rs) implements the hidden reporters. `hook report` consumes a bounded JSON payload from stdin; `hook serve` adapts persistent stdio tool calls to the same report. They return the provider-neutral response rather than making an approval decision or adding text to the prompt. A transport deadline bounds delivery to 250 ms. Envelope size and per-route queue limits bound memory use; these are admission guarantees, not evidence of a native runtime pass.

The daemon validates reports and admits them into [`session/hook_queue.rs`](../../crates/runner-daemon/src/session/hook_queue.rs). [`HookFeed`](../../crates/runner-daemon/src/session/hook_feed.rs) wraps that in-memory `HookReceiver` and drains provider payloads. There is no report NDJSON file or separate per-event payload file. The Codex parser validates session and turn identities, rejects stale and subagent reports, and may read Codex's transcript for outcomes that its hooks alone cannot settle.

### Example: submitting a prompt

When you submit a prompt, Codex reports `UserPromptSubmit` through the configured helper. For a valid new turn, `CodexParser::current_hook` produces `AgentEvent::TurnStarted`, wrapped in a runtime batch. This status path is separate from the bytes that render the terminal.

```mermaid
sequenceDiagram
    participant C as Codex CLI
    participant H as Bundled hook helper
    participant Q as Daemon HookRoutes / queue
    participant W as CodexStatusWatcher
    participant P as PTY monitor
    participant M as SessionManager / SessionModel
    participant U as AppStore

    C->>H: UserPromptSubmit payload
    H->>Q: Bounded typed HookReport
    P->>W: drain_events(...)
    W->>Q: Drain admitted reports
    W-->>P: AgentEvent::Batch(TurnStarted)
    P-->>M: RuntimeOutput::AgentEvent
    M->>M: Reduce session event and run effects
    M-->>U: Publish session status
```

`idle_monitor_thread` calls `watcher.drain_events`, passing pending cancel information and callbacks. Accepted events go through the runtime output channel. Events needing feedback wait for the manager's response without holding the detector lock; this lets the shared reducer control deferred transcript work.

The forwarder in [`session/manager/output.rs`](../../crates/runner-daemon/src/session/manager/output.rs) calls `publish_model_event_for_launch` with `SessionEvent::Agent`. The shared [`session/state/`](../../crates/runner-daemon/src/session/state/) reducer owns status, attention, drafts, turn outcomes and conversation identity. The manager executes publication and persistence effects after reduction. The adapter interprets Codex evidence; it does not independently write UI status.

## What this refactor leaves for later

#777 reorganized agent knowledge while preserving behavior, apart from its documented model-chooser Enter fix. It did not itself centralize status decisions. The later [#791 reducer](../features/archive/791-session-state.md) now owns shared session state, while adapters still interpret their own provider events. The [runtime integration checklist](../arch/runtime-integration.md) records what a new runtime must prove beyond fitting this code structure.

## Reading path

Follow one fresh Codex chat before tackling resume and status precedence:

1. Read `RuntimeAdapter` and the `adapter` / `for_key` registry in [`runtimes/mod.rs`](../../crates/runner-daemon/src/runtimes/mod.rs).
2. Read `first_turn_argv`, `resume_plan`, and `launch_args` in the [`Codex` adapter](../../crates/runner-daemon/src/runtimes/codex/mod.rs).
3. Follow `spawn_runtime_direct` into `spawn_direct_inner`, then inspect `apply_runtime_args`, in [`session/manager/spawn.rs`](../../crates/runner-daemon/src/session/manager/spawn.rs).
4. Read `SpawnSpec` and the `SessionRuntime` trait in [`session/runtime.rs`](../../crates/runner-daemon/src/session/runtime.rs), then `PtyRuntime::spawn` in [`session/pty_runtime.rs`](../../crates/runner-daemon/src/session/pty_runtime.rs).
5. Trace output back through [`session/manager/output.rs`](../../crates/runner-daemon/src/session/manager/output.rs). For the path from those bytes to painted pixels, continue with [terminal rendering](terminal-rendering.md).

The [runtime integration checklist](../arch/runtime-integration.md) describes the complete requirements for adding another agent once these boundaries are familiar.
