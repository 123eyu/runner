# Claude Code accounts

Open **Settings → Agents → Claude Code → Accounts** to manage separate Claude Code sign-ins. Your existing native sign-in appears as **Personal**. Runner stores account names, folders and selections; Claude Code keeps each sign-in in its own configuration folder and credential store.

## Add an account

Choose **Add account**, enter a name and leave the folder on Auto, or use **Browse** to select an existing account folder. Auto derives a folder such as `~/.claude-work` from the name. Choose **Add and log in** and complete Claude Code’s browser sign-in. An existing signed-in folder is recognized without signing in again. Runner completes initial setup for an added signed-in account so its first interactive chat opens past Claude Code’s onboarding. During sign-in, **Open sign-in page** reopens the page and **Cancel** stops the attempt.

Each account shows its sign-in status, email and any available organization or plan. An account can be signed in without a plan label.

## Switch the active account

Choose **Use** beside a signed-in account, or select it from the Claude Code dropdown in the usage popover. The dropdown lists signed-in accounts and their weekly usage. **Manage accounts…** opens the Claude Code settings page. The popover and footer show the active account’s usage.

The selected account shows **Active**, including when it is also **main**; both badges appear together.

The active account applies to new direct chats, role chats, crew members, resumed chats and recovery after a restart. Running sessions keep their original account. With several accounts configured, a Claude Code pane shows its account after the title, and its side panel shows **Account** and the `config` folder.

A role or launch environment that explicitly sets `CLAUDE_CONFIG_DIR` keeps that pin. An explicit `~/.claude` pin is distinct from the original native sign-in, which leaves the variable unset.

## Resume and fork

A paused chat says which account it ran on and which active account will continue it. **Resume** continues the same saved conversation; switching accounts does not copy or move its history. A fork starts a separate conversation in the active account’s folder. Existing chats retain their recorded history location even after an inactive account is forgotten.

Starting a new conversation with Claude Code’s `/clear` uses the account running that process. After resuming a Personal conversation with Work, `/clear` starts a new conversation in Work’s folder, and subsequent Resume follows that new history. Selecting another active account while the process is running does not change this location.

A setup that has never added another account keeps the existing native launch, resume, fork and usage behavior, without an account chip or dropdown.

## Rename, make main or remove

An account’s **⋯** menu offers **Rename**, **Make main**, **Reveal in Finder** (**Show in Explorer** on Windows) and **Remove**. Renaming leaves its folder unchanged. Main identifies the owner of shared configuration; changing main never changes an account’s sign-in target.

The main account, active account and original native account cannot be removed. Removing another account forgets it in Runner and preserves its folder, files and sign-in on disk. Switch away from an active account before removing it.
