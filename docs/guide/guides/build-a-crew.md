# Build a crew

Create reusable roles, put them into a crew, and write a clear handoff between them. A coder and a reviewer are enough for a first team.

## Create the roles

Open **Roles** and create two roles. Choose an installed, signed-in agent for each. You can use one runtime for both or mix providers; each needs its own working installation.

| Role | Suggested instructions |
| --- | --- |
| Coder, handle `coder` | Implement the requested change, match local patterns, run relevant checks, and fix review findings. Stay within the goal. |
| Reviewer, handle `reviewer` | Read the brief and working-tree diff. Report must-fix defects with file and line references, or state that the review is clean. Do not edit files. |

If these handles already exist, use the saved roles or choose different handles. Keep task-specific requirements out of the role prompt so it also makes sense in a direct chat.

## Assemble the crew

1. Open **Crews**, create a crew and give it a name.
2. Add the coder and reviewer roles. Give their slots distinct handles.
3. Set the coder as **Lead**. A launchable crew needs exactly one lead.
4. Review the runtime, model and effort for each slot. Leave values inherited unless this crew needs an override. If you change the runtime, recheck model and effort for that runtime.
5. Add team conventions, for example:

   ```text
   The coder implements; the reviewer waits for a handoff. Before editing, the coder checks repository instructions and existing changes. The coder sends the reviewer a summary, changed files and checks run. The reviewer checks the working-tree diff without editing. Repeat until the review is clean. Leave all changes uncommitted unless the mission goal explicitly authorizes commits or a pull request. Ask the person if a decision changes scope.
   ```

Team conventions apply to every member in a mission. Role prompts describe responsibilities; conventions describe how those responsibilities fit together.

## Give the crew a goal

Start the mission with a specific outcome, the working directory, checks to run, and any permission to commit, push or publish. For example: “Add a regression test for the reported parsing bug, fix it, and have the reviewer verify both. Leave the changes uncommitted.”

Runner does not isolate working directories automatically. If the team should work in a Git worktree, create it before launch and select its path. Avoid assigning overlapping edits to several workers without a clear owner.

## Use the examples

The repository includes ready-to-copy prompts and conventions:

- [Pair coding](https://github.com/yicheng47/runner/tree/main/examples/pair-coding): the seeded coder/reviewer loop.
- [Codex trio](https://github.com/yicheng47/runner/tree/main/examples/codex-trio): coder, reviewer and QA.
- [Development crew](https://github.com/yicheng47/runner/tree/main/examples/dev-crew): architect, implementer and reviewer.
- [Documentation crew](https://github.com/yicheng47/runner/tree/main/examples/docs-crew): architect, writers and editor.
- [More examples](https://github.com/yicheng47/runner/tree/main/examples): game and role-playing crews.

Copy the role prompts into roles and the conventions into the crew. The example goals are starting points to replace with your task. The links above show the current examples on `main`; use the repository's tag selector for examples matching an older release.

See [Your first mission](../getting-started/first-mission.md) to launch and follow the team.
