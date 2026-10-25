<!-- LOGO -->
<h1 align="center">
  <img src="assets/icon.png" alt="Runner" width="128" />
  <br />
  Runner
</h1>

<p align="center">
  <a href="https://github.com/yicheng47/runner/stargazers"><img src="https://img.shields.io/github/stars/yicheng47/runner?style=flat-square&logo=github&label=stars" alt="GitHub stars" /></a>
  <a href="https://github.com/yicheng47/runner/releases"><img src="https://img.shields.io/github/downloads/yicheng47/runner/total?style=flat-square&label=downloads" alt="下载量" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/github/license/yicheng47/runner?style=flat-square&cacheSeconds=86400" alt="许可证" /></a>
  <a href="https://discord.gg/6BrfZdHKNf"><img src="https://img.shields.io/badge/Discord-join%20community-5865F2?style=flat-square&logo=discord&logoColor=white" alt="加入 Runner Discord" /></a>
  <a href="#社区"><img src="https://img.shields.io/badge/WeChat-user%20group-07C160?style=flat-square&logo=wechat&logoColor=white" alt="微信用户群" /></a>
  <a href="https://runnersh.dev/downloads/"><img src="https://img.shields.io/badge/macOS%20%7C%20Windows-native-2ea44f?style=flat-square" alt="macOS 与 Windows" /></a>
  <a href="https://afdian.com/a/runnersh"><img src="https://img.shields.io/badge/%E8%B5%9E%E5%8A%A9-%E7%88%B1%E5%8F%91%E7%94%B5-946BE7?style=flat-square" alt="通过爱发电赞助 Runner" /></a>
</p>

<p align="center">
  <a href="./README.md">English</a> · 简体中文
</p>

<p align="center">
  <strong>让终端 agent 协同工作。</strong>
  <br />
  Claude Code、Codex、Antigravity CLI、Copilot CLI 和 pi 在同一个 mission 里做同一件事。每个 agent 在真实终端里保留自己的 TUI，Runner 是夹在它们中间的那一层。
</p>

<p align="center">
  <a href="https://runnersh.dev/"><strong>官方网站</strong></a>
  ·
  <a href="https://runnersh.dev/downloads/"><strong>下载 Runner</strong></a>
</p>

<p align="center">
  <img src="assets/hero.png" alt="Runner 起始页 — 在同一个工作区新建聊天、终端或 crew mission" width="100%" />
</p>

<p align="center">
  <a href="#功能">功能</a>
  ·
  <a href="#演示">演示</a>
  ·
  <a href="#支持的-agent">Agent</a>
  ·
  <a href="#让你的-agent-来驱动-runner">CLI</a>
  ·
  <a href="#示例-crew">Crew 示例</a>
  ·
  <a href="#社区">社区</a>
  ·
  <a href="./CONTRIBUTING.md">参与贡献</a>
</p>

> Alpha · 原生支持 macOS（Apple Silicon）和 Windows（x64）。

Runner 让命令行编码 agent 通过可复用的 crew、真实终端和共享 mission feed 一起工作。会话和历史记录都保存在你的机器上。

## 功能

<table>
  <tr>
    <td width="50%" valign="middle">
      <a href="assets/crews.png"><img src="assets/crews.png" alt="Crew 详情页展示 coder 和 reviewer 槽位、lead 标记、单槽位配置覆盖和共享团队约定" width="100%" /></a>
    </td>
    <td width="50%" valign="middle">
      <h3>可复用的 Crew</h3>
      <p>把保存的角色组合成 crew，指定一个 lead，并设定共享的团队约定。查看各槽位使用的 agent 和模型，按需覆盖配置，之后随时用同一支团队发起 mission。</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="middle">
      <a href="assets/mission_feed.png"><img src="assets/mission_feed.png" alt="mission feed 中 coder、reviewer 和 QA 在协作，一个问题正等待你回答" width="100%" /></a>
    </td>
    <td width="50%" valign="middle">
      <h3>Mission 与协作</h3>
      <p>每个 mission 为 crew 的各个槽位提供实时终端和共享 feed，agent 在这里协作，需要你决定的问题也会出现在这里。单个槽位可以单独停止、恢复或重启，不影响其他槽位。<a href="./docs/arch/arch.md">架构 →</a></p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="middle">
      <a href="assets/mission_terminal.png"><img src="assets/mission_terminal.png" alt="mission 里一个槽位自己的终端：coder 的 Codex 会话，旁边是 crew 的会话卡片" width="100%" /></a>
    </td>
    <td width="50%" valign="middle">
      <h3>真实终端</h3>
      <p>每个 agent 都在真实 PTY 里保留自己的 TUI，跑在 GPU 绘制的 <code>alacritty_terminal</code> 网格上：鼠标上报、输入法（包括拼音）、文件路径粘贴和点击打开，每个 chat 和 mission 下面还有一个终端抽屉。</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="middle">
      <a href="assets/chat_split.png"><img src="assets/chat_split.png" alt="一个标签页分成四栏：Codex、Claude Code、Antigravity CLI 和 pi 并排运行" width="100%" /></a>
    </td>
    <td width="50%" valign="middle">
      <h3>分栏与多窗口</h3>
      <p>标签页可以一直分栏到窗口放不下为止，拖动各栏重新排列，把标签页归进文件夹，用 <code>⇧⌘N</code> 或 <code>Ctrl+Shift+N</code> 打开更多窗口。</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="middle">
      <a href="assets/notifications.png"><img src="assets/notifications.png" alt="Settings → Notifications：选择哪些 chat 和 mission 事件发送桌面通知" width="100%" /></a>
    </td>
    <td width="50%" valign="middle">
      <h3>桌面通知</h3>
      <p>Runner 在后台时，agent 需要输入、完成一轮回复或失败都可以发通知，点击后打开对应的 chat 或 mission。在 <strong>Settings → Notifications</strong> 中选择通知事件；chat 通知默认开启，mission 通知需要手动开启。</p>
    </td>
  </tr>
</table>

**更多功能：**

- **Runner 关闭后 agent 照样工作** — 会话跑在 Runner 的后台服务 `runnerd` 里。退出应用或者应用崩溃，重新打开后会接回同一批正在运行的终端；停止的 agent 会在下次启动时各自接着原来的对话。
- **在一个地方管理 MCP 服务与技能** — **Settings → MCP** 和 **Settings → Skills** 按 agent 列出它自己的配置，可以按 agent 单独关掉某一项，只改动你碰过的那一条。
- **项目** — 绑定一次工作目录；在项目里发起的 chat 和 mission 会继承它的 cwd，并在侧边栏里归在一起。
- **浅色与深色** — Carbon 和 Runner Light，外加 Catppuccin，每种模式各有一套应用配色和终端配色。Claude Code 会实时跟着切换。

## 演示

快速展示可复用角色、crew 配置、终端分屏，以及 mission 中 coder、reviewer 和 QA 的协作。

https://github.com/user-attachments/assets/c2209015-1497-403a-98bd-cdc5502bee47

## 支持 Runner

如果 Runner 对你有帮助，欢迎赞助项目的持续开发与维护。

- [![通过爱发电赞助 Runner](https://img.shields.io/badge/%E8%B5%9E%E5%8A%A9%20Runner-%E7%88%B1%E5%8F%91%E7%94%B5-946BE7?style=for-the-badge)](https://afdian.com/a/runnersh) — 推荐中国大陆用户使用，通过微信支付或支付宝付款。
- [![通过 Ko-fi 赞助 Runner](https://img.shields.io/badge/%E8%B5%9E%E5%8A%A9%20Runner-Ko--fi-FF5E5B?style=for-the-badge)](https://ko-fi.com/yichengwang) — 推荐海外用户使用，通过 PayPal 付款。

## 社区

- 加入 [Runner Discord](https://discord.gg/6BrfZdHKNf)，交流使用问题、工作流、Bug 和代码贡献。
- Bug 和功能建议：[GitHub Issues](https://github.com/yicheng47/runner/issues)。
- 感谢 [LINUX DO](https://linux.do) 社区提供交流与开源推广平台。
- 扫码加入 Runner 微信用户群。群二维码 7 天过期，如果扫码提示失效，请[提一个 issue](https://github.com/yicheng47/runner/issues/new) 提醒我更新。

<img src="assets/wechat_group_qr.png" alt="Runner 微信用户群二维码" width="200" />

## 让你的 agent 来驱动 Runner

Runner 里的一切也都是一条 `runner` 命令，你的 agent、你的脚本和终端前的你都能用它驱动 Runner。你日常用的 agent 可以规划好一个修复，交给一个 coder 加 reviewer 的 crew 去实现，然后继续干自己的事，而它拉起的每个会话仍然是一个你随时可以打开查看的真实终端。

在应用之外驱动一整个 mission：

```sh
mission=$(runner mission start --crew <crew> --goal-file - -q < brief.md)
runner mission feed "$mission" --follow                   # 实时查看 crew 的消息
runner msg post --mission "$mission" --to <lead> "message"
runner mission answer "$mission" <question_id> <choice>   # 回答 crew 问你的问题
runner mission stop "$mission"
```

| 命令 | 用途 |
| --- | --- |
| `runner mission` | 启动、跟踪、回答、停止、恢复和归档 mission |
| `runner crew`、`runner role`、`runner project` | 创建和编辑 crew、角色和项目 |
| `runner chat start`、`runner session` | 发起 chat；停止、恢复、重启或归档任意会话 |
| `runner msg`、`runner signal`、`runner ask` | mission 内 crew 成员之间的沟通 |
| `runner status`、`runner daemon` | 查看和停止 Runner 的后台服务 |

`runner help` 列出全部命令。加上 `--json` 输出机器可读的结果，agent 用的就是这种。后台服务没有运行时，命令会自己拉起它，所以应用关闭时命令照样可用。

**安装。** 在 macOS 上，如果登录 `PATH` 已包含 `~/.local/bin`，Runner 会在首次启动时把 `runner` 安装到那里；如果 `PATH` 中的 `/usr/local/bin` 可写，则安装到后者；否则去 **Settings → General → Command line** 点一下即可。在 Windows 上，Runner 会把它加入用户 `PATH`。agent 不需要额外设置：Runner 会为每个检测到的 agent 安装 `runner` skill，让它读取与当前版本一致的 `runner help agents` 指南；同一个设置区域里的 **Runner skill for agents** 开关可以关掉它。

## 支持的 Agent

Runner 集成了 **Codex、Claude Code、Antigravity CLI、GitHub Copilot CLI、pi、Cursor、Grok Build 和 OpenCode v2**。**TRAE CLI** 可按需启用。各 agent 的功能和平台支持有所不同。

[Agent 安装、功能对比与平台说明 →](./docs/guide/reference/supported-agents.md)

## 示例 Crew

全新安装会预置 **Pair coding crew**：coder 负责实现，reviewer 审查 diff，反复迭代直到审查通过。两个角色默认使用 Codex，也可以分别选择其他 agent。

[组建 Crew 并查看示例提示词 →](./docs/guide/guides/build-a-crew.md)

## 致谢

- **[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)** 与 **[gpui-pre](https://github.com/longbridge/gpui-kit)** — UI 使用 Zed GPU 加速 UI 框架的 crates.io 快照。感谢 Zed 团队构建并开源了 GPUI，感谢 gpui-kit 和 Jason Lee（huacnlee）发布 gpui-pre；Zed 的终端 crate 是 Runner 终端分层的架构参考。
- **[alacritty_terminal](https://github.com/alacritty/alacritty)** — 每一栏底下的终端网格、解析器和回滚。
- **[xterm.js](https://github.com/xtermjs/xterm.js)** — 程序化绘制的制表符字形表按其 MIT 声明转录自 WebGL 插件（`crates/runner-app/LICENSE.xterm`）。
- **[Windows Terminal ConPTY](https://github.com/microsoft/terminal)** — Windows 构建按 MIT 声明内置了微软的 `conpty.dll` 和 `OpenConsole.exe`（`crates/runner-app/LICENSE.conpty`）。
- **[Sparkle](https://sparkle-project.org)** — macOS 的更新器。

## 作者

Runner 由 **王逸成**（Yicheng Wang / Jason Wang）编写和维护，GitHub 账号 [@yicheng47](https://github.com/yicheng47)。

## 许可证

MIT。Copyright (c) 2026 Yicheng Wang (Jason Wang)。你可以将 Runner 用于任何用途，包括工作场景和闭源产品，并可自由修改和再分发，只需保留版权声明（见 `LICENSE`）。2026-08-22 至 2026-09-22 期间发布的版本以 GPL-3.0-only 许可证发布，并继续以该许可证提供。
