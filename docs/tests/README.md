# Validation docs

Use these documents to select checks and preserve their evidence. [Test Scope](../../AGENTS.md#test-scope) governs ordinary changes: name the changed behavior, choose checks that cover it, and explain the connection before running them. A documentation sweep does not run the app or establish a new product pass.

| Location | Purpose |
| --- | --- |
| [regression/README.md](regression/README.md) | Maintained live cases for the current product, with stable IDs, applicability and evidence requirements. |
| [full-smoke-test.md](full-smoke-test.md) | Execution procedure: candidate identity, development-app isolation, native control, bounded fixtures and cleanup. |
| [runs/](runs/) | Dated results for a particular candidate, platform and attempt. New records use `YYYY-MM-DD-<platform>.md`; existing filenames remain stable. |
| [archive/](archive/) | Historical feature validation, investigations and earlier smoke records, including failed and unverified checks. |
| This directory | Feature-specific validation plans or records while their work is active or awaiting a release sweep. |

For an ordinary feature, select relevant regression cases and add feature-specific checks. Use smoke-tier rows for an explicitly requested release smoke and all active rows for an explicitly requested complete regression or a major refactor requiring that coverage. Reusing the execution procedure does not expand a feature's test scope.

Write new run results separately from maintained cases. Record commands and exit codes, candidate identity, selected case instances, observations, evidence and gaps. Preserve failures and later retries; a source review, CLI probe, CI pass or historical user confirmation does not establish a current native pass.

At a release sweep, move shipped feature records to `archive/`, promote lasting current live checks into `regression/` under its [promotion rules](regression/README.md#promotion-and-maintenance), and repair references. Automated-only assertions and one-time investigations stay with their owning tests or records. Retain validation evidence even when a mission brief or obsolete proposal is pruned.
