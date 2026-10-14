import { Plugin } from "@opencode/plugin";

export async function setup(context) {
  const model = context.options.model;
  if (!model?.variant) return;
  // Runner launches a private server; sessions from before it started belong
  // to native history and must retain their existing selection.
  const createdAfter = Date.now() - process.uptime() * 1000;
  let owner;
  let selection;
  let applied = false;
  const registration = await context.session.hook("prompt", async ({ sessionID }) => {
    if (applied || (owner && owner !== sessionID)) return;
    const session = await context.session.get({ sessionID });
    if (session.parentID || session.time.created < createdAfter) return;
    const messages = await context.session.context({ sessionID });
    if (messages.length) return;
    owner ??= sessionID;
    // Admission precedes model resolution, including the TUI's --prompt.
    // Persist the native variant once, so later TUI choices and resume win.
    selection ??= context.session.switchModel({ sessionID, model }).catch((error) => {
      selection = undefined;
      throw error;
    });
    await selection;
    applied = true;
  });
  return () => registration.dispose();
}

export default Plugin.define({ id: "runner.opencode.status", setup });
