# #1031 — Roles and crews over the companion protocol

Shipped in [0.18.1](https://github.com/yicheng47/runner/releases/tag/v0.18.1) through [PR #1033](https://github.com/yicheng47/runner/pull/1033). The results below retain the original candidate and failures; archiving adds no new live-test pass.

The candidate is the working-tree diff on `feat/1031-remote-role-crew-edit`, based on `3da540053f4364ea90f3437f3f41d12fd4c6f8c6`. Implementation and review run in `.worktrees/feat-1031-remote-role-crew-edit` with its own Cargo target directory. The [feature spec](../../features/archive/1031-remote-role-crew-edit.md) defines the contract.

## Selected checks

| Changed behavior | Check and reason |
| --- | --- |
| Thirteen new requests and role/crew results | Core golden round trips, named missing/mistyped field errors, nullable omissions, permission enum fallback, and request ID helpers exercise the independent wire contract. |
| Role/crew/slot writes use host operations | Real-host relay tests read back all editable fields, clear nullable fields, verify stable role handles, reorder slots, transfer/remove the lead, delete crews with their slots, and delete unused roles. |
| Desktop refresh events | Each successful write checks the exact role/crew/slot event names and null payload; crew deletion checks both events. Failed writes emit no change event. |
| Safe failures | Real-database host action tests check exact codes and fixed messages for handle collisions and all three role-delete refusals, malformed values, unknown IDs, invalid permutations, and generic operation failure. |
| Permission-mode inference | Host reads exercise Claude AcceptEdits, Codex Bypass, legacy Trae Auto falling back to Default, and a runtime with no modes. Null updates preserve sent args; explicit modes rewrite them; null create mode uses Default. |
| Environment privacy | A role with a secret environment value is read remotely and updated; results exclude the environment, incoming environment fields are ignored, and the stored environment survives. Request Debug and malformed-field errors exclude private canaries. |
| Compatibility and CLI boundary | An unserved host returns unknownRequest for every new request; negotiated features include all action names; generic_call_has_no_remote_requests still iterates the complete allowlist. |
| Shared desktop inference and boxed action storage | App tests plus workspace Clippy cover the extracted helper and the internal payload representation; companion goldens verify unchanged serialization. |

No live regression rows are selected. No development app is launched: this mission changes protocol and host operations, which the in-process tests cover, and introduces no native controls. Live checks are **not run**, as required by the mission.

## Execution

- The initial core/daemon/CLI nextest run: **1,823 passed, 1 failed, 2 skipped**. The unchanged `usage::tests::codex_rpc_handshake_and_timeout` failed with `CodexNoAnswer` at its five-second handshake deadline during concurrent compilation/testing. Its isolated rerun passed (one test).
- Final `cargo nextest run --locked -p runner-core -p runner-daemon -p runner-cli -p runner-app --cargo-profile ci --no-fail-fast -j 4`: **2,774 passed, 5 skipped**, including the initially failing usage handshake test.
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`: **passed**.
- `cargo fmt --all --check` and `git diff --check`: **passed**.
- Logs for this local run: `/tmp/runner-1031-tests.log`, `/tmp/runner-1031-timeout-retry.log`, `/tmp/runner-1031-tests-final.log`, and `/tmp/runner-1031-clippy-final.log`.
