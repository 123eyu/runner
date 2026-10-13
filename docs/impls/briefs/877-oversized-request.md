# 877 — an oversized request fails alone instead of dropping the connection

Fix [#877](https://github.com/yicheng47/runner/issues/877) (bug, P2). Jason asked on 2026-10-12 for a codex duo mission that ends in an open PR, not merged. Starting this mission is his go for everything in it.

Work only in `/Users/jason/repos/yicheng47/runner/.worktrees/fix-877-oversized-request`, on `fix/877-oversized-request`, cut from `origin/main` at `7fd40b9c`. Its tip is this brief. Do not create another branch or checkout, touch the root checkout or another worktree, or share another worktree's target directory. If main moves and you need it, rebase onto `origin/main`; never merge main into the branch.

## The bug, verified against `7fd40b9c`

- `crates/runner-core/src/protocol/socket.rs:356`: `SocketTransport::send` queues any frame for the writer thread without checking its size.
- `crates/runner-core/src/protocol/socket.rs:306`: the writer thread treats every `frame.write` error as a broken connection and calls `owner.disconnect(...)`, including the local "frame too large" error from `Frame::encode` (`crates/runner-core/src/protocol/wire.rs:87`). Every pending call fails, the app sees `daemon/disconnected`, and repeats can show the repeated-crash banner while `runnerd` is healthy.
- `crates/runner-core/src/protocol/api.rs:163`: `session_paste_image` carries the image as `Vec<u8>`, which JSON encodes at about 3.6 bytes per byte, so on macOS (`crates/runner-app/src/terminal_paste.rs:11`, `prepare_image_paste`) a clipboard image over about 18 MB trips it.
- The response direction has the same failure: `crates/runner-daemon/src/daemon/server.rs:761` queues a `RESPONSE` frame of any size, and `write_frame` (`server.rs:585`) fails on `encode()` and ends the connection.

## The fix

1. **Client.** `SocketTransport::send` refuses a frame whose encoded length exceeds `wire::MAX_FRAME` before queueing it, returning a `ClientError` that names the request size and the limit in MB (for example "request too large for runnerd (75.3 MB, limit 64 MB)"). Nothing is queued, the transport stays open, and `call` still removes its pending entry. This covers `call`, `attach`, `input` and `resize`, since they all go through `send`. Keep the limit in one place: if a helper on `Frame` reports the encoded length or the check, have `encode` use it too. The writer thread keeps disconnecting on real write errors.
2. **Daemon.** At `server.rs:761`, a `RESPONSE` frame over `MAX_FRAME` is replaced by a `REQUEST_ERROR` for that call id with a readable message, so the caller gets a `ClientError` and the connection keeps serving. Leave events, the hook path and the handshake as they are.
3. **Image paste.** With 1 in place, `prepare_image_paste` returns the error, and its callers (`crates/runner-app/src/surfaces/chat.rs:1514`, `crates/runner-app/src/surfaces/mission_workspace/input.rs:272`) already show it in the pane. Check that this holds and change nothing there unless it does not. Windows keeps sending no image bytes (#875).

Out of scope: chunking or a binary encoding for image bytes, rejecting TIFF before the round trip, the CLI's one-shot client (`crates/runner-cli/src/client.rs`), and raising `MAX_FRAME`. Name any of them as a follow-up in the PR if review thinks it worthwhile.

## Tests

- In `socket.rs`'s `mod tests`, following the pattern of `protocol_errors_reach_pending_calls_as_restart_messages`: a call whose request encodes past `MAX_FRAME` returns the size error at once (not at the request deadline), `is_closed()` stays false, no `daemon/disconnected` event is sent, and a small call on the same transport then succeeds.
- In the daemon: a response over `MAX_FRAME` reaches the client as a `REQUEST_ERROR` for its id and the next request on that connection is served. If a real oversized response is awkward to produce, factor the response-frame construction into a small function and test that, plus one test that the connection survives.
- Keep test allocations just over `MAX_FRAME`, not multiples of it; Windows CI is load-sensitive. Imports, statics and helpers used only by `#[cfg(unix)]` tests get `#[cfg(unix)]` themselves; macOS clippy cannot catch this, and Windows CI will.

## Validation

Run each, log it to a file, and read the exit code from the command itself, never through `tail` or `grep`:

- `cargo nextest run --locked --workspace --no-fail-fast --cargo-profile ci`
- `cargo clippy --locked --workspace --all-targets --profile ci -- -D warnings`, and again with `--features updater`
- `cargo fmt --all --check`
- `git diff --check`

## Boundaries

This crew has no QA slot. Neither slot runs the development app, launches agents or reproduces the bug live; the live check (paste an image over 18 MB into a macOS chat pane and see the error in the pane with the connection intact) is left for Jason and named in the PR. No slot starts, stops, restarts or types into Jason's production Runner, its chats or missions; this mission itself runs there. Never open a Runner database. No extra agents, crews or subagents.

## Review, verification and authorization

The coder owns implementation, tests and fixes. The reviewer waits for an explicit Runner handoff, then reviews the whole diff against this brief, must-fix first with file:line pointers. A finding is must-fix when an oversized request or response can still close the connection or fail other pending calls, when the caller waits for the deadline instead of getting the size error at once, when a real write error no longer disconnects, when the size limit is computed in more than one place, or when the tests do not prove the connection survives. Other edge cases go in the handoff as follow-ups.

Iterate through Runner until the reviewer posts `NO REMAINING MUST-FIX ISSUES`.

After the clean review, Jason authorizes:

- **One commit** on top of current `origin/main`, this brief folded in, with a subject naming the change (for example `fix(core): refuse an oversized request without dropping the connection`) and no co-author trailers.
- **Push** with `git push -u origin fix/877-oversized-request`.
- **Open the PR** with `gh pr create --base main`. The body says `Closes #877` and carries the summary, the changed behaviours with the checks that cover them and their exit codes, follow-ups from review, the live check left for Jason, and no agent session links.
- **Watch CI** with `gh pr checks <n> --watch` until nothing is pending, on macOS and Windows. Fold each fix into the commit (`git commit --amend`), have the reviewer check it, and push with `git push --force-with-lease`.

**Do not merge**: Jason said on 2026-10-12 that the mission ends at the open PR with CI green, and he merges. Do not delete the branch or worktree, or cut a nightly or release.

The final handoff goes to everyone through Runner: the PR URL and CI result, what changed, the checks with exit codes, the reviewer's verdict, follow-ups, and the live check left for Jason. Then both slots stand by.
