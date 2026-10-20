# Filtered message read feedback

Tracking issue: [#965](https://github.com/yicheng47/runner/issues/965).

## Motivation

An agent can handle messages returned by `runner msg read --since` or `--from` while leaving its unread inbox untouched. The growing unread count then looks like ignored work. The command should explain the distinction where the agent encounters it.

## Scope

After a successful filtered read, report the unread inbox count and an actionable hint on stderr if unread messages remain. Count all unread messages in the caller's inbox from the captured batch, independent of query filters, respecting the membership boundary and latest valid acknowledgement. Show the hint even when the filter selects no messages. A filtered read of an already-consumed inbox remains quiet.

Preserve stdout, successful exit status, and existing acknowledgement semantics: filters never acknowledge messages, unfiltered reads acknowledge only successfully printed inbox messages, and messages arriving after capture remain unread. This change does not introduce a JSON response format for `msg read` or track repeated queries in the daemon.

Generated worker, team and lead prompts, the installed agent skill, CLI help and the user guide explain that default reads already track the inbox, filtered reads are inspection tools, and receipt-only chat replies are distinct from consuming the inbox.

Affected guide pages: `docs/guide/guides/drive-runner-from-agents.md` and generated `docs/guide/reference/cli.md`. No native UI changes.

## Implementation phases

1. Derive the filtered-read unread count from the existing inbox projection after folding acknowledgements, and emit the hint on stderr.
2. Update guidance, CLI help, documentation and affected golden expectations.

## Verification

- Message-read unit tests cover the filter matrix, membership boundaries, stale cursor and log fallback, valid acknowledgements, failed output and arrivals after capture.
- A CLI roundtrip test proves the hint stays on stderr, counts messages hidden by filters, appears for empty filtered results, never emits an acknowledgement, and disappears after an unfiltered read.
- Prompt tests and spawn goldens verify generated guidance; CLI goldens and `make guide-cli` cover help and the generated reference.
- Format checking and Clippy cover the affected Rust crates. The message-read subset of `MIS-MSG-07` is covered through isolated CLI fixtures; live agent delivery and native Windows checks are outside this CLI-feedback change.

## Authorization

Jason requested inline implementation followed by a Runner review mission using Claude Opus 5.5 at xhigh effort, then authorized merging and review-mission cleanup on 2026-10-20.
