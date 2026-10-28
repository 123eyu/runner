# 项目

项目为工作目录赋予名称，并在侧边栏中组织该目录下的聊天和任务。这样就不用每次启动时重复选择同一个文件夹。

侧边栏默认显示项目。搜索旁的列表筛选按钮可切换到活动视图，汇总各项目中运行的聊天与终端、具有活动成员的任务，以及需要关注的工作；切换视图不会改变项目组织方式。

从侧边栏创建项目并选择目录。在该项目中启动的新聊天和任务会使用此目录。启动时仍可另行指定工作目录。

CLI 也能创建相同的关联：

```sh
runner project create "My app" --path /absolute/path/to/my-app
runner chat start --runtime codex --project "My app"
runner mission start --crew "Pair coding crew" --project "My app" --goal "Review the README for accuracy."
```

Windows 上请使用绝对 Windows 路径，例如 `C:\Users\you\src\my-app`。包含空格的路径要加引号。

## 目录与分组

在 Shell 中，从已注册项目的目录内启动聊天或任务，会自动关联到该项目。`--project` 显式选择项目及其目录；`--cwd` 则直接指定启动目录。这两个参数不能同时使用。CLI 的目录默认为 Shell 当前目录，因此启动任务前请确认所在位置。

项目只是目录关联，并非沙箱或隔离的 Git 检出。多个会话可以编辑相同文件。若要在独立 worktree 中工作，请先创建 worktree，再从其路径启动。

将现有任务移动到另一个项目只改变侧边栏分组；不会移动仓库文件，也不会更改运行中进程的目录。

## 移除项目

删除项目会归档其成员，不会删除磁盘目录。若仍有成员运行，CLI 会拒绝删除，除非提供 `--force`。删除分组前请先停止并检查受影响的工作。

项目命令见 [CLI 参考](../reference/cli.md#runner-project)。
