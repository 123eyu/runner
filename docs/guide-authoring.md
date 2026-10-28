# Writing the user guide

The user guide lives in [`guide/`](guide/README.md). It describes shipped application behavior for people using Runner; architecture, feature specs and validation records remain outside it.

## Format and navigation

- Use plain GitHub-flavored Markdown: no front matter, MDX or raw HTML. Code fences may contain literal command placeholders.
- Start each page with one `#` title and a summary paragraph. The website uses these as the page title and description.
- Keep one prose paragraph per line, separated by blank lines. Preserve the structure of lists, tables, alerts and code fences.
- Maintain the four-section nested link lists in `guide/README.md` and `guide/zh-CN/README.md`. They define publication and navigation order; add every public page once in each locale, in the same order.
- Link between guide pages with relative `.md` paths, including heading fragments when useful. The website rewrites these to routes. Link outside the guide with an absolute URL so a website build does not need contributor docs or example files; state when an example link points to `main`.
- Use GitHub alerts for material notes, tips or warnings. Both locales use the same alert markers such as `[!WARNING]`; translate the alert prose.
- Use the app's labels and vocabulary. Give macOS shortcuts first, then Windows, and state differences where the action is described.
- Verify against source and the relevant release. Distinguish implemented capabilities from native validation, especially for newer runtimes and Windows. Never describe a code inspection or automated check as a fresh-install walkthrough.

## Bilingual maintenance

English sources live directly under `guide/`; canonical Simplified Chinese sources live under `guide/zh-CN/` with identical relative paths. Each locale has one H1 and a summary paragraph per page. Translate all linked articles and index labels; preserve list/table structure, GitHub alert markers and executable examples. Keep product, runtime, model and theme names, user handles, flags and literal CLI output unchanged. Use the app's `crates/runner-app/i18n/zh-hans.toml` for labels, with bilingual labels when useful for navigation. Guide vocabulary is role → 角色, crew → 小队, mission → 任务, session → 会话, lead → 队长, pane → 窗格 and tab → 标签页.

Relative links in a Chinese article resolve within `zh-CN/`; update heading fragments to the translated heading. Absolute website links should retain the reader's locale. When moving or adding a page, update both indexes together. Run `python3 script/check-guide-translations.py` for source/navigation and per-heading paragraph parity, local links and anchors, and literal code-fence preservation. Review the translation against the English source as well; structural checks do not establish translation accuracy.

## Generate the CLI reference

From the repository root on macOS:

```sh
make guide-cli
make guide-cli-check
```

These targets build the branch's CLI with the `ci` profile and use Python 3 to generate or check both `guide/reference/cli.md` and `guide/zh-CN/reference/cli.md`. No running app, daemon or signed-in agent is required. The generator invokes only `help`, `help agents` and public `--help` paths. It discovers nested commands from the help output rather than maintaining a second command list; hidden hook commands stay internal. Output uses LF line endings and removes trailing spaces emitted by the help formatter, while preserving indentation and command text.

On Windows, run the commands separately in PowerShell:

```powershell
cargo build --locked -p runner-cli --profile ci
if ($LASTEXITCODE -ne 0) { throw 'CLI build failed' }
python script/generate-cli-reference.py --runner target/ci/runner-agent-cli.exe
if ($LASTEXITCODE -ne 0) { throw 'CLI reference generation failed' }
python script/generate-cli-reference.py --runner target/ci/runner-agent-cli.exe --check
if ($LASTEXITCODE -ne 0) { throw 'CLI reference is stale' }
```

CI checks both committed pages against the built binary on both platforms. The generator translates only the Chinese page title, summary, note and navigation; every fenced help block is identical to the generated English output. Do not use an installed `runner` from another version or edit either generated page by hand. Update the generator for Chinese wrapper changes. An explicit `--output` generates/checks only the requested English file, preserving the temporary-file stale-detection workflow. To verify stale detection without touching the committed file, copy it to a temporary path, change the copy, and run the generator with `--check --output <temporary-path>`; the command must fail and leave the copy unchanged.

## Behavior changes and publication

A PR changing a user-visible command, setting, shortcut or screen updates the relevant English and Simplified Chinese guide pages in the same diff. Feature specs name their affected pages. Check page links and heading fragments, render the Markdown, and review command examples against the current CLI help.

The website integration belongs to the website repository. Each default build resolves one full commit SHA from Runner's `main` and reads both guide locales from that same revision, with each index as navigation. It does not read a release tag. A correction on `main` reaches the website on its next build. English routes remain under `/docs/`; Simplified Chinese routes mirror them under `/zh-CN/docs/`. The website may use a validated full-SHA build-time override to review an unmerged guide PR; never pin production to an unmerged branch or fetch guide content at runtime.
