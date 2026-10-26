# 1031 — Roles and crews over the companion protocol

Archived after shipping in [0.18.1](https://github.com/yicheng47/runner/releases/tag/v0.18.1) through [PR #1033](https://github.com/yicheng47/runner/pull/1033). Retained for the companion feature-negotiation, whole-record update and environment-privacy contracts.

> Tracking issue: [#1031](https://github.com/yicheng47/runner/issues/1031). Related: [#912](https://github.com/yicheng47/runner/issues/912) (the iPhone app, its first client), [#836](https://github.com/yicheng47/runner/issues/836) (the companion protocol; see its [archived spec](836-web-remote-control.md#remote-actions)).

## Motivation

A paired client can start a chat from a host's role and a mission from a host's crew, but it cannot change either. The companion protocol reads them only through `catalog`, which carries names and choices but no prompts, commands or slot overrides, and no request creates, edits or deletes them: when remote actions were designed, editing roles and crews stayed on the host (Jason, 2026-10-23). The iPhone app ([#912](https://github.com/yicheng47/runner/issues/912)) needs to manage them, so the protocol and the host serve that now, before any client uses it (Jason, 2026-10-26).

## Scope

Decided by Jason on 2026-10-26:

- **Protocol and host only.** `runner-core`'s companion types and fixtures, and `runnerd`'s companion adapter. The desktop's remote window keeps its read-only Roles and Crews lists; letting it edit is a separate follow-up, as m7c followed m7b.
- **Every field the desktop's role editor shows, except the environment.** A role crosses the wire with its name, handle, runtime, command, args, working folder, system prompt, model, effort, speed and permission mode. A paired client can already type anything into the host's shells, so withholding command and args would protect nothing. Environment variables can hold API keys, the desktop editor does not edit them, and they never cross the wire.

The rules the existing remote actions follow carry over unchanged:

- **One operation, two callers.** Each request runs the `runnerd` operation the desktop's Roles and Crews pages run, so handle rules, lead hand-off and delete refusals are the desktop's, and the change emits the same `role/changed`, `crew/changed` and `slot/changed` events, so the host's own open pages refresh at once.
- **Features.** Each request is listed in `welcome.features`. A host without one answers `unknownRequest`, and a client hides the control rather than failing on use. The protocol stays at version 1.
- **The result is not the state.** A create returns the new item's id, every other write returns null, and the client reads the item again with `role` or `crew`, or refreshes `catalog`. No new push: the overview does not carry roles, and a client re-reads when a page opens and after its own writes.
- **An update replaces the editable record.** The client sends every field, as the desktop's edit forms do, and the last write wins. The wire has no field whose absence means "keep".
- **Fixed diagnostics.** A refusal the desktop would show reaches the client as a fixed message that names the reason without echoing supplied text or listing host items; anything else keeps the generic failure.

Out of scope: role environment variables, crew import and export ([#804](https://github.com/yicheng47/runner/issues/804)), editing a running mission's roster, any desktop UI, and any new CLI command.

## Wire

All requests carry a numeric `id`. Nullable fields are written as `null` and may be omitted, as in the existing actions. Field names are camelCase; ids are the host's role, crew and slot ids, the same ones `catalog` returns.

### Reads

- `role` takes `role`. It returns the record: `id`, `handle`, `name`, `runtime`, `command`, `args`, `workingDir`, `systemPrompt`, `model`, `effort`, `speed` (`standard`, `fast` or null) and `permissionMode` (`default`, `acceptEdits`, `auto`, `bypass`, or null for a runtime that offers no modes, read from the role's args as the desktop editor reads it).
- `crew` takes `crew`. It returns `id`, `name`, `conventions` (the crew's system prompt addendum, or null) and `slots` in position order, each with `id`, `handle`, `role`, `lead` and its overrides `runtime`, `model`, `effort` and `speed`, null where the slot inherits from its role.

### Roles

- `createRole` takes `handle`, `name`, `runtime`, `command`, `args`, `workingDir`, `systemPrompt`, `model`, `effort`, `speed` and `permissionMode`. A null `command` uses the runtime's default command on the host; a null `permissionMode` is `default`, as the desktop's create form sends. It returns `{"role":"<id>"}`.
- `updateRole` takes `role` and the same fields except `handle`, which never changes after creation, as on the desktop. A null `permissionMode` keeps `args` exactly as sent; a mode rewrites the runtime's permission flags in them. It returns null.
- `deleteRole` takes `role` and returns null. As on the desktop, it is refused while a crew has a slot for the role, or while the role has unarchived chats or missions.

### Crews and slots

- `createCrew` takes `name` and `conventions`, returning `{"crew":"<id>"}`. `updateCrew` takes `crew`, `name` and `conventions`, returning null. `deleteCrew` takes `crew` and returns null, removing its slots with it, as on the desktop.
- `addSlot` takes `crew`, `role`, `handle`, `runtime` and `model`, as the desktop's Add slot dialog does, and returns `{"slot":"<id>"}`. A crew's first slot leads it. Effort and speed are set afterwards with `updateSlot`, as the desktop's slot popup does.
- `updateSlot` takes `slot`, `handle`, `runtime`, `model`, `effort` and `speed`, and returns null. A null override inherits from the role; a shell runtime is refused, as on the desktop.
- `removeSlot` takes `slot` and returns null. Removing the lead hands the lead to the first remaining slot.
- `reorderSlots` takes `crew` and `order`, the crew's complete slot order as ids, and returns null. `setLead` takes `slot` and returns null.

### Errors

- `notFound` for a role, crew or slot id the host does not have.
- `badRequest` for a malformed field: a handle the desktop's rule rejects, an empty name, an unknown runtime, speed or permission mode, a shell slot runtime, or an `order` that is not exactly the crew's slots.
- `failed` with a fixed message for each refusal the desktop shows: a role or slot handle already in use, and a role still used by crews, by unarchived chats or by unarchived missions. The message says what to do on the host or in the catalog; it does not list the crews, chats or missions.
- `failed` with the generic message for anything else.

Each request and each read's result has a golden file under `runner-core/src/companion/fixtures/`.

## Privacy

No environment value appears in a result, a fixture or a log line. The adapter does not log request bodies, which now carry system prompts, and `ActionRequest`'s redacted `Debug` stays redacted. The new requests stay unreachable through `runner call`, as every remote request is.

## Guide

[Remote access](../../guide/guides/remote-access.md)'s trust paragraph says editing roles and crews stays on the host. With this change a paired device can do it through the protocol, so the paragraph treats paired computers as trusted with roles and crews too, while the remote window's own description still says this Runner's remote window shows them read-only.

## Verification

- **Wire,** in `runner-core`: each new request and result round-trips through its golden file; each request's missing or mistyped field names that field in its `badRequest`; an unknown request still answers `unknownRequest`.
- **Host,** in `runnerd`, against a real host database as the existing action tests run: each request's success and its effect as the desktop's own request leaves it, including the emitted events; `role` and `crew` reading back what a write stored, with `permissionMode` inferred from args; each refusal with its fixed message and code; an unknown id; and that no result carries `env`, using a role whose environment is set.
- **Features:** every new request is in `ActionRequest::NAMES` and `welcome.features`, and a client against a host without them sees `unknownRequest`.
- **No live checks.** Nothing a person sees or operates in the running app changes; the in-process tests exercise the whole path.
