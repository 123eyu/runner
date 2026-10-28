# 第一个任务

用默认的结对编程小队完成一个小任务：coder 负责实现，reviewer 检查差异，双方的对话显示在同一条共享消息流中。

## 准备小队和文件夹

全新安装时，Runner 会预置 **Pair coding crew** ，成员为 **@coder** 和 **@reviewer** 。两者均使用 Codex，因此请先安装并登录 Codex，在[聊天](first-chat.md)中确认它能正常工作。打开 **小队（Crews）** ，选择 **Pair coding crew** ，检查两个成员。已有角色或小队的安装可能不会收到预置配置；[组建小队](../guides/build-a-crew.md)介绍了如何创建同样的组合或选择其他智能体。

第一次任务请选择一个小型仓库或可丢弃的项目。Runner 会在你指定的目录中启动小队。它不会替你创建隔离副本或 Git worktree；所有成员都能查看并修改该目录。

> [!WARNING]
> 任务中的智能体具有广泛访问权限：支持的运行时使用 Bypass permissions，pi 则使用受信任的工作区。智能体可以编辑文件和运行命令，而无需直接聊天中可能出现的逐次批准。请选择你愿意交给小队处理的文件夹和目标。

## 启动任务

1. 按 **⇧⌘M** / **Ctrl+Shift+M** ，或从小队页面启动任务。
2. 选择 **Pair coding crew** 。对话框会显示成员列表和智能体设置。
3. 输入标题，例如“介绍项目”。
4. 输入具体的 **目标（Goal）** ，例如：

   ```text
   Create a short PROJECT-TOUR.md explaining the top-level folders in this repository. Inspect the files before writing. The reviewer should check every claim against the repository and send corrections to the coder. Make no code changes, commits, pushes or pull requests. Leave the final diff for me to inspect.
   ```

5. 选择 **项目（Project）**，或选择 **无项目（No project）** 将任务归入最近记录。检查下方的 **工作目录（Working directory）**：选择项目会填入其目录，目录位于其他项目内时，离开字段后项目选择会随之更新。点击 **启动任务（Start mission）**，或按 **⌘Enter** / **Ctrl+Enter**。

每个成员都会收到目标、角色指令和团队约定。coder 开始实现；reviewer 按约定等待 coder 交接。目标可以留空，以便先建立任务，但第一次尝试时在启动前写好目标更容易跟进。

## 跟进工作

**动态（Feed）** 显示消息和协调事件。打开某个成员的终端可查看其完整智能体界面，再返回消息流跟进团队。 **⌘[** / **Ctrl+[** 和 **⌘]** / **Ctrl+]** 可在消息流和已打开的会话标签页之间循环切换。

使用任务的消息输入框向小队补充修正或约束。在开头输入 `@` 并选择成员，即可添加收件人标签；重复操作可选择多人。点击标签可将其移除。没有收件人标签或开头提及的消息会广播给所有成员；定向消息只会唤醒收件人。如果智能体向你提问，问题卡片会显示可选答案，请在卡片上选择。普通的智能体审批提示如有出现，会显示在智能体终端中，而非消息流的问题卡片。

预置约定要求 coder 与 reviewer 反复审查，直到没有必须修复的问题。“已完成”的消息本身不会停止进程。请自行检查最终差异与报告的验证结果。

## 停止与归档

完成后使用任务的 **停止（Stop）** 控制。 **恢复（Resume）** 会让已停止的成员继续原有对话。也可以通过成员自己的控制区单独停止或恢复某个成员，而不影响其他成员。

不想让任务继续显示在活动侧边栏时，可以将其归档。已归档任务位于 **设置 → 已归档（Settings → Archived）** ；恢复归档会重新打开任务，但成员仍处于停止状态。之后使用 **恢复（Resume）** 继续对话。等效 CLI 命令如下：

```sh
runner mission list
runner mission stop <mission-id>
runner mission archive <mission-id>
```

将 `<mission-id>` 替换为列表中的 ID。停止或归档任务不会撤销小队对文件的修改。

接下来：[任务如何工作](../concepts/missions.md)，或[组建自己的小队](../guides/build-a-crew.md)。
