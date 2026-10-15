# Writing the user guide

The user guide lives in [`guide/`](guide/README.md). It describes shipped application behavior for people using Runner; architecture, feature specs and validation records remain outside it.

## Format and navigation

- Use plain GitHub-flavored Markdown: no front matter, MDX or raw HTML. Code fences may contain literal command placeholders.
- Start each page with one `#` title and a summary paragraph. The website uses these as the page title and description.
- Keep one prose paragraph per line, separated by blank lines. Preserve the structure of lists, tables, alerts and code fences.
- Maintain the four-section nested link list in `guide/README.md`. It defines publication and navigation order; add every public page there once.
- Link between guide pages with relative `.md` paths, including heading fragments when useful. The website rewrites these to routes. Link outside the guide with an absolute URL so a website build does not need contributor docs or example files; state when an example link points to `main`.
- Use GitHub alerts for material notes, tips or warnings. Keep the first version text-only and English-only; Chinese translation follows [#565](https://github.com/yicheng47/runner/issues/565).
- Use the app's labels and vocabulary. Give macOS shortcuts first, then Windows, and state differences where the action is described.
- Verify against source and the relevant release. Distinguish implemented capabilities from native validation, especially for newer runtimes and Windows. Never describe a code inspection or automated check as a fresh-install walkthrough.

## Generate the CLI reference

From the repository root on macOS:

```sh
make guide-cli
make guide-cli-check
```

These targets build the branch's CLI with the `ci` profile and use Python 3 to generate or check `guide/reference/cli.md`. No running app, daemon or signed-in agent is required. The generator invokes only `help`, `help agents` and public `--help` paths. It discovers nested commands from the help output rather than maintaining a second command list; hidden hook commands stay internal. Output uses LF line endings and removes trailing spaces emitted by the help formatter, while preserving indentation and command text.

On Windows, run the commands separately in PowerShell:

```powershell
cargo build --locked -p runner-cli --profile ci
if ($LASTEXITCODE -ne 0) { throw 'CLI build failed' }
python script/generate-cli-reference.py --runner target/ci/runner-agent-cli.exe
if ($LASTEXITCODE -ne 0) { throw 'CLI reference generation failed' }
python script/generate-cli-reference.py --runner target/ci/runner-agent-cli.exe --check
if ($LASTEXITCODE -ne 0) { throw 'CLI reference is stale' }
```

CI checks the committed page against the built binary on both platforms. Do not use an installed `runner` from another version or edit the generated page by hand. To verify stale detection without touching the committed file, copy it to a temporary path, change the copy, and run the generator with `--check --output <temporary-path>`; the command must fail and leave the copy unchanged.

## Behavior changes and publication

A PR changing a user-visible command, setting, shortcut or screen updates the relevant guide pages in the same diff. Feature specs name their affected pages. Check page links and heading fragments, render the Markdown, and review command examples against the current CLI help.

The website integration belongs to the website repository. Its contract is to read `docs/guide/` from the latest published `v*` release tag, with the index as navigation. A correction on `main` appears on GitHub immediately and reaches that release-based website on the next release. Do not add a README link to `runnersh.dev/docs` until that route is live.
