# Projects

A project gives a working directory a name and groups its chats and missions in the sidebar. It saves you from choosing the same folder for every launch.

The sidebar defaults to Projects. The list-filter button beside Search switches to Active, which gathers running chats and terminals, missions with live members, and work needing attention across projects; switching views preserves your project organization.

Create a project from the sidebar and choose its directory. New chats and missions started in that project use its directory. You can still supply a different working directory when starting work.

In **Start a chat** and **Start mission**, check the **Project** field above **Working directory** before starting. It starts with the active project, or **No project** when none is active; a project's **+** menu starts with that project. **No project** files the work under Recents and initially leaves the directory blank. An empty pane's chat stays with its tab's project, so its Project field is locked.

Choosing a project fills its folder unless the directory is already inside it, such as a worktree. Choosing **No project** keeps the current directory. When a typed directory loses focus or Browse returns a folder inside another project, the selection follows it; nested projects use the deepest match. A directory outside every project keeps the choice and shows an amber hint. You can still start there; the work files under the project shown.

The CLI can create the same association:

```sh
runner project create "My app" --path /absolute/path/to/my-app
runner chat start --runtime codex --project "My app"
runner mission start --crew "Pair coding crew" --project "My app" --goal "Review the README for accuracy."
```

On Windows, use an absolute Windows path such as `C:\Users\you\src\my-app`. Quote paths containing spaces.

## Directory and grouping

From a shell, a chat or mission started inside a registered project's directory is associated with that project automatically. `--project` explicitly selects a project and its directory; `--cwd` supplies a launch directory instead. These two flags cannot be combined. The CLI's directory defaults to the shell's current directory, so check where you are before starting a mission.

A project is a directory association, not a sandbox or an isolated Git checkout. Several sessions can edit the same files. To work in a separate worktree, create that worktree first and launch with its path.

Moving an existing mission into another project changes its sidebar grouping; it does not move repository files or change a running process's directory.

## Removing a project

Deleting a project archives its members; it does not delete the directory on disk. The CLI refuses deletion with running members unless `--force` is supplied. Stop and review the affected work before deleting the grouping.

See [CLI reference](../reference/cli.md#runner-project) for project commands.
