export default function (pi: any) {
  let enabling = false;

  function getToolNames(tools: unknown): string[] {
    if (!Array.isArray(tools)) return [];
    return tools
      .map((tool) => {
        if (typeof tool === "string") return tool;
        if (tool && typeof tool === "object" && typeof (tool as { name?: unknown }).name === "string") {
          return (tool as { name: string }).name;
        }
        return null;
      })
      .filter((name): name is string => Boolean(name));
  }

  async function ensureMermaidToolsActive() {
    if (enabling) return;
    enabling = true;
    try {
      const allToolNames = getToolNames(pi.getAllTools());
      const desiredTools = ["render_mermaid"].filter((name) => allToolNames.includes(name));
      if (desiredTools.length === 0) return;

      const activeTools = getToolNames(pi.getActiveTools());
      const missingTools = desiredTools.filter((name) => !activeTools.includes(name));
      if (missingTools.length === 0) return;

      await pi.setActiveTools([...activeTools, ...missingTools]);
    } finally {
      enabling = false;
    }
  }

  pi.registerCommand("reload-runtime", {
    description: "Reload extensions, skills, prompts, and themes",
    handler: async (_args: unknown, ctx: any) => {
      await ctx.reload();
    },
  });

  // Keep reload as a slash command only. sendUserMessage() bypasses slash-command
  // expansion, so a reload_runtime tool would recurse instead of reloading.
  pi.registerCommand("enable-mermaid", {
    description: "Enable the render_mermaid tool for this session",
    handler: async (_args: unknown, ctx: any) => {
      await ensureMermaidToolsActive();
      ctx.ui.notify("render_mermaid enabled for this session", "info");
    },
  });


  pi.on("session_start", async () => {
    await ensureMermaidToolsActive();
  });

  pi.on("turn_start", async () => {
    await ensureMermaidToolsActive();
  });

  pi.on("session_tree", async () => {
    await ensureMermaidToolsActive();
  });

  pi.on("session_branch", async () => {
    await ensureMermaidToolsActive();
  });
}
