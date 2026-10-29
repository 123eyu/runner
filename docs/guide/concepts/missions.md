# Missions

A mission holds agents working on one goal, their terminals, and a shared record of messages and questions. Start from a saved crew, or create an empty mission and add members as needed.

## Goal and membership

For a crew mission, Runner starts a session per slot and delivers the goal to every member, together with their role instructions, team conventions and coordination instructions. Members act on their own part and wait for any required handoff. The mission captures its own roster, so the crew remains a reusable template.

With **No crew**, the mission starts without sessions. Add a role from the mission and give it a task. Each joins as a member and receives the shared mission context. The person controlling the mission can add or remove any member while it runs. Removing a member stops its session but keeps the mission's history.

The mission's working directory is shared by its members when supplied. Runner does not make branches, worktrees or copies of your repository. Set up any isolation you need before launch and describe it in the goal.

## Feed and terminals

The feed records messages, questions, answers and lifecycle events in a persistent event log. A member can address another member by handle, post to the crew, or ask the person for a decision. Runner delivers coordination messages to the relevant agents; their own terminals retain the detailed work and output.

Answer a question through its feed card. For an ordinary follow-up, use the message composer. Type `@` at the start and pick a member to add a recipient chip; repeat to choose several. Click a chip to remove it, or press Backspace in an empty draft to remove the last chip. With no chips or leading mentions, the message broadcasts to the crew. Only its recipients receive inbox notifications. Mentions inside the text remain text. You can also [use the CLI](../guides/drive-runner-from-agents.md) to read the feed, send a message or answer a pending question.

An unsent draft in an agent's terminal holds inbox delivery and shows **Inbox waiting**. Submitting the draft or clearing the composer lets delivery resume; submitted text still visible in Codex or Antigravity history does not hold it. Runner follows the current composer when terminal redraws move it, as Antigravity and Claude Code do. If delivery remains held after you verify the composer is empty, use **Submit / clear** to reset input tracking.

An inbox notice that Runner typed itself is not a draft. If the agent ignored its Return, the slot shows **Message not submitted — press Return**. Runner presses Return on that notice again when the agent's turn ends or a prompt closes, and otherwise about every two minutes. It never types a second notice over the first one. If a member's delivery stays held for more than two minutes while it has unread messages, Runner adds one warning to the feed that names the member and the reason. `runner mission show` lists that warning under RECENT WARNINGS. A member that is still starting, which can last through its whole first turn, gets no such warning.

The scrollbar on the feed’s right edge shows your position in long histories. Drag its thumb, click its track, or scroll over it to move through the feed. New messages keep the feed at the bottom while you are near the end; while reading older messages, use **New messages** to return to the bottom.

## Permissions

Mission launches use the runtime's broad-access mode, generally **Bypass**; pi uses a trusted workspace. A role's more restrictive direct-chat permission setting does not turn a mission into an approval-by-approval workflow. Write clear limits in the goal, and choose the working directory accordingly.

## Stop, resume and archive

- **Stop a slot** to halt just that agent. **Resume** continues its saved conversation; **Restart** starts that mission slot with a fresh conversation.
- **Stop the mission** to halt its members. **Resume** resumes stopped sessions.
- **Archive** stops active members and removes the mission from active lists. Its feed remains available under **Settings → Archived**. **Restore** reopens it with all members stopped. Use **Resume** to continue their saved conversations. The feed records each reopen.

While a mission is paused, its unread-response green dot is hidden in the sidebar and collapsed groups. Unread responses remain saved, and the dot returns after resuming if any remain unread. Viewing a role's session still marks that session's response as read while paused. Error and approval indicators keep their usual behavior.

A “done” message or an idle agent does not automatically mean all sessions have exited. Stopping and archiving never revert repository changes.

Closing Runner can leave missions working in its background service. **Settings → General → When Runner quits** controls that choice. See [Sessions](sessions.md) for relaunch and recovery behavior.
