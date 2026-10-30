# 1044 — Finish host setup with a PIN, and explain an unavailable host before pairing

> Tracking issue: [#1044](https://github.com/yicheng47/runner/issues/1044).
> Related: [#1041](https://github.com/yicheng47/runner/issues/1041), the same paused host after pairing. Both issues use the neutral offline wording below.
> Design: every state reuses an existing row, dialog or button in `runner-app`; no new component is introduced. This spec is the detailed plan the issue's `needs-design` label calls for. Source inspected at `6dfc734a`.

## Motivation

A host can be paired only after it has a PIN, but signing in never asks for one. The host shows **On through the relay**, and the only hint is the PIN row's subtitle further down Settings → Remote access. The other computer finds out only after someone types six digits and submits them: "{computer} has no PIN yet."

When **Allow remote access** is off, the host closes its relay connection and is offline to its other computers. The picker still opens PIN entry for it. Submitting reaches the relay, which closes the channel with code 4409 (`offline`). `PairState::refusal` maps that to `Failed`, shows "Could not pair… submit the PIN again", clears the field and focuses it. Every retry fails the same way, so a correct PIN looks rejected. Timeouts and relay errors take the same path.

## Decisions

The issue's open questions, settled for this fix:

1. **Where the PIN prompt goes: both.** The app opens the existing Set PIN dialog right after a sign-in it started, and the remote access status shows a setup row until a PIN exists. `runner remote login` prints the next step.
2. **The listing stays as it is.** `Computer` carries `online` and `last_seen_at`. A paused host drops its relay connection, so the relay cannot tell paused from offline, and nothing tells another computer whether a host has a PIN. Reporting either one needs a new host report and a `ListAccountHosts` field in `runner_server`, which is out of scope here. Before pairing, the other computer relies on `online` and on the relay's 4409 close. A missing PIN is still discovered on submission; the host-side prompt is what prevents it.
3. **Shared wording with #1041.** Before pairing the reason is never known, so this fix uses only the neutral "{computer} is offline", which #1041 also uses for an unknown reason. The detail can list turning remote access on among the things to check, without claiming it is off. #1041 owns the known-reason sentence ("Remote access was turned off on {computer}"); this fix does not add it.

## Scope

### Host: the PIN is part of setup

**Prompt after sign-in.** When a sign-in started from this window's Settings (`remote_connect`, from Account or Remote access) completes, meaning the status leaves `Enrolling`/`Confirming` for `Connecting` or `Connected`, and the host has no PIN, open the existing Set PIN dialog (`remote_open_pin`) in that window. The dialog is a page-level Settings overlay, so it opens from either pane. If the window has left Settings in the meantime, do not prompt; the setup row covers it.

- Tie the prompt to the sign-in transition this window observed, not to `pin_set == false` alone. Never prompt on app start, daemon reconnect, resuming from **Paused**, reconnecting, or in another window. Do not act on a trust snapshot published before the daemon restored its secret store (see `warn_pin_lock`).
- The dialog is unchanged: same title, explanation, two steps, weak-PIN warning and Cancel. Cancel closes it and the setup row stays.

**Setup row.** While the host is signed in, has no PIN and PIN entry is not locked, the status card shows a setup row directly below its header and above the account details. It reuses the `pin_locked` row treatment in `remote_pairing`: a `SettingsRow` with the warning tint, a warning-coloured `key-round.svg` label icon, and a primary **Set PIN** button that calls `remote_open_pin`.

| Key | English | zh-hans |
| --- | --- | --- |
| `remote.pin_setup` | Finish setup: set a PIN | 完成设置：设置 PIN |
| `remote.pin_setup_detail` | Set a PIN so your other computers can pair with this one. Runner’s service never sees it. | 设置 PIN，让其他电脑可以与这台电脑配对。Runner 服务永远不会看到此 PIN。 |

While the setup row is shown, the PIN row in the Pairing section is hidden, so there is one **Set PIN** action. Once a PIN exists, the setup row disappears and the PIN row returns with **Change PIN**. The setup row also shows while **Paused** or **Reconnecting**: the PIN is still missing. The header and **Allow remote access** are unchanged.

**CLI.** After `runner remote login` prints "Connected as @…", read the trust snapshot. If no PIN is set, print one more line: `Set a PIN so your other computers can pair with this one: runner remote pin`. Login still exits 0. `runner remote status` already shows `PIN  not set`.

### Other computer: offline is not a wrong PIN

All entry points go through `RemoteClient::open_computer_pair`: the picker (`choose_computer`), **Pair** in the Settings computers list, and **Pair** / **Pair again** on the remote workspace's unavailable card. Implement the offline check there once.

**Known offline before PIN entry.** If the listed computer has `online == false`, open the dialog in a new **Offline** state instead of `Entry`.

- Title "{computer} is offline", with the `monitor.svg` icon in muted colour. Detail: "Runner can pair once {computer} is back online. Check that it’s awake, signed in, and has **Allow remote access** on." The hint line shows the listing status from `status_label`, for example "Offline · seen 5 min ago".
- There is no PIN field. Buttons: **Cancel** and primary **Try again**.
- **Try again** refreshes the computer listing. When a refreshed listing shows the computer online while the dialog is still **Offline** with no complete PIN, move to `Entry` and focus the field. Apply this in `sync_computer_pair` too, so a refresh from anywhere moves the dialog. If the computer is still offline, stay. Do not add polling.

| Key | English | zh-hans |
| --- | --- | --- |
| `client.pair_offline_title` | {computer} is offline | {computer} 已离线 |
| `client.pair_offline_detail` | Runner can pair once {computer} is back online. Check that it’s awake, signed in, and has Allow remote access on. | {computer} 重新上线后即可配对。请确认它没有休眠、已登录，并且已打开“允许远程访问”。 |

**Try again** reuses the existing `remote.try_again` label (重试).

**Submission fails for a reason other than the PIN.** Classify the daemon's error in `PairState::refusal`, keep the typed PIN, and never present the failure as a PIN to re-enter.

| Daemon message contains | State | What the dialog shows |
| --- | --- | --- |
| `offline (4409)` (`route::close_error`) | **Offline** | The Offline state above, with the typed PIN kept but hidden. **Try again** submits the kept PIN again, which is one new attempt; if that fails as offline, the dialog stays **Offline**. |
| `Pairing timed out` (`route::pair`) | **TimedOut** | The PIN layout with the field still filled and enabled; hint in warning colour: "{computer} didn’t respond in time. Try again." Primary **Try again** submits it again. |
| `Remote service is unavailable`, `Remote route refused`, `Remote route closed` other than 4409, `Could not list computers` | **Unreachable** | As for TimedOut, with the hint "Couldn’t reach {computer} through the relay. Try again." |
| `wrongPin`, `pinLocked`, `noPin` | unchanged | Unchanged. Only `wrongPin` clears the field, marks it invalid and shows "That PIN didn’t match." |
| anything else | **Failed** | As for TimedOut, with the hint "Could not pair with {computer}. Try again." |

| Key | English | zh-hans |
| --- | --- | --- |
| `client.pair_timed_out` | {computer} didn’t respond in time. Try again. | {computer} 未及时响应。请重试。 |
| `client.pair_unreachable` | Couldn’t reach {computer} through the relay. Try again. | 无法通过中继连接 {computer}。请重试。 |
| `client.pair_failed` (replaced) | Could not pair with {computer}. Try again. | 无法与 {computer} 配对。请重试。 |

- In these states the primary button reads **Try again** and is focused, so Enter retries. Editing the field is still allowed; if the PIN becomes incomplete, the button is disabled as it is for `Entry`.
- "Keep the PIN" means keeping the existing `PinField` entity with its value, re-enabled and not invalid, instead of replacing it with a new empty field. A changed listed identity keeps its current behaviour: the dialog resets to `Entry` with a new empty field and the new fingerprint.
- Each **Try again** is one explicit attempt, as Pair is today. Nothing retries automatically.
- Classification stays on the existing message tokens, as `refusal` and `Connection::failure` already work. Do not change `ClientError`, the client protocol or daemon messages. Because of the CLI line, the PR is not `frontend-only`.

## Out of scope

- A `paused` or `pin_set` field in the account listing, and any `runner_server` change. Filing that as a follow-up is Jason's call.
- #1041's paired-host reconnecting banner, its unavailable workspace and its known-reason wording.
- CLI pairing errors from `runner remote pair`.
- Unused `remote.step_*` intro copy; nothing renders it.

## Implementation phases

1. Host: the sign-in transition prompt, the setup row replacing the PIN row while no PIN is set, and the CLI login line, with tests.
2. Other computer: the Offline gate in `open_computer_pair`, the new dialog states and copy, `sync_computer_pair` recovery, and the kept PIN, with tests.
3. Guide pages and copy in English and zh-hans.

## Verification

Automated, in `runner-app` (the existing `settings::remote` and `remote_client` test modules) and `runner-cli`:

| Changed behaviour | Check |
| --- | --- |
| Prompt after sign-in | Simulated status `Enrolling → Connected` with `pin_set: false` after `remote_connect` in this window opens the PIN dialog; the same with `pin_set: true` does not; `Paused → Connected`, `Reconnecting → Connected` and a first status after startup do not. |
| Setup row | Signed in with no PIN: setup row present, PIN row absent, **Set PIN** opens the dialog. PIN set: setup row absent, PIN row shows **Change PIN**. PIN locked: the existing locked row and PIN row, no setup row. |
| Offline gate | `open_computer_pair` with `online: false` from each entry point opens **Offline**, with no PIN field focused. A listing update showing it online moves the dialog to `Entry` and focuses the field. |
| Failure classification | `PairState::refusal` maps each daemon string in the table, using the exact text the daemon produces, including `close_error(4409)`'s. |
| Kept PIN | After TimedOut, Unreachable, Failed and submit-time Offline, the field keeps its six digits, is enabled and not invalid, and **Try again** sends exactly one new `remote_pair` with the same PIN. After `wrongPin` the field is cleared and invalid. |
| CLI login line | Login completion prints the PIN line when `pin_set` is false and not when it is true. |

Run the `runner-app` and `runner-cli` tests, workspace Clippy with warnings denied, `cargo fmt --all --check` and `git diff --check`. CLI help does not change, so `make guide-cli` is not needed.

Live checks, for Jason, with two computers on one account in English and Chinese. They extend [REMOTE-PAIR-01](../tests/regression/remote.md):

1. Sign in a fresh host from Settings: the Set PIN dialog opens once sign-in completes. Cancel it: the setup row shows under the status header and the PIN row is hidden. Set a PIN from the row: the row disappears and **Change PIN** returns.
2. Pause the host, then pick it on an unpaired computer: the dialog says it is offline and has no PIN field. Turn remote access back on and choose **Try again**: PIN entry appears.
3. Open PIN entry while the host is online, pause the host, then submit the correct PIN: the dialog says it is offline, and after resuming the host, **Try again** pairs with the kept PIN.
4. A wrong PIN still shows "That PIN didn’t match" and clears the field.

## Affected documentation

- [Remote access](../guide/guides/remote-access.md) and its [zh-CN copy](../guide/zh-CN/guides/remote-access.md): the PIN prompt after sign-in and the setup row (“Through Runner’s relay”, “Who can reach this computer”); the offline dialog and kept PIN on failures (“Pair and open another computer in the app”); the login line (“Connect from a terminal”).
- [REMOTE-PAIR-01](../tests/regression/remote.md) gains the offline and setup expectations at the next release sweep, not in this PR.
