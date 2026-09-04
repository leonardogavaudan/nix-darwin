# AGENTS.md (personal profile)

Profile-specific instruction layer for personal machines.

<!-- BEGIN SHARED -->
## Personal Profile Notes

- Active agent profile: `personal`.
- This layer is merged after `agents/shared/AGENTS.md`.
<!-- END SHARED -->

<!-- BEGIN CLAUDE -->
## Claude-Specific Notes

- No additional Claude-only instructions yet.
<!-- END CLAUDE -->

<!-- BEGIN CODEX -->
## Codex-Specific Notes

- For browser automation, prefer the local `agent-browser` CLI over Browser MCP.
- Attach to the real Brave Default profile via CDP on port `9223`:
  - Check availability with `brave-cdp status 9223`.
  - Attach with `agent-browser connect 9223`.
  - If CDP is not running, `brave-cdp launch-default 9223 --force-quit` relaunches Brave on the real Default profile.
- Avoid launching isolated browser profiles unless the task explicitly needs a clean session.
- For Google services, prefer the local `gws` CLI/MCP tools over Codex Google plugins or app connectors.
- The Gmail Codex plugin is intentionally uninstalled; do not reinstall or suggest it unless the user explicitly asks.
- `gws` has two registered Google accounts:
  - Personal: `leonardogavaudan@gmail.com`
  - Work (Algolia): `leonardo.gavaudan@algolia.com`
- Select the `gws` account explicitly with `GOOGLE_WORKSPACE_CLI_ACCOUNT` for each operation. Use the personal account when the request has no clear work context, and the work account for Algolia/work requests.
- If the intended Google account is still ambiguous, ask before any send, write, delete, or other state-changing operation.
<!-- END CODEX -->

<!-- BEGIN PI -->
## Pi-Specific Notes

- Pi has internet/browser skills available:
  - `exa-search` for primary web search and content extraction (**preferred**).
  - `brave-search` as fallback web search when Exa is unavailable.
  - `browser-tools` for interactive browsing and JavaScript-heavy pages.
- **Search preference:** Prefer Exa (`exa-search` skill or Exa MCP) over Brave for web searches unless Exa is unavailable or unsuitable for the task.
- **Multi-agent browser safety:** multiple agents may run at the same time.
  - Assume shared browser state; do not use the default "last tab" navigation flow.
  - Always open a dedicated working tab with `browser-nav.js <url> --new` before navigating.
  - Do not close, redirect, or interact with tabs you did not create for the current task.
  - Before any click/type/submit action, verify the active tab URL/title matches your current task.
<!-- END PI -->
