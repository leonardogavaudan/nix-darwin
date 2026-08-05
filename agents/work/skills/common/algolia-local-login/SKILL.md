---
name: algolia-local-login
description: Sign into a local AlgoliaWeb frontend backed by beta-dashboard using Leonardo's Brave Browser Pi profile, the 1Password browser extension, and a macOS Keychain-protected unlock secret. Use when starting or visually testing AlgoliaWeb locally without a local Rails backend, especially when localhost redirects, 1Password unlocks, or repeated login steps are slowing down UI QA.
---

# Algolia Local Login

Run a frontend-only AlgoliaWeb preview against the beta backend, then sign in on `localhost` without exposing credentials in commands or tool output.

## Start the frontend

From the worktree's `_client` directory, expose Vite on localhost:

```bash
DASHBOARD_WEB_BACKEND_HOST=https://beta-dashboard.algolia.com yarn vite --host localhost --port <port>
```

Do not start the local Rails backend unless the user explicitly asks for it.

## Sign in

1. Read and use the `browser-pi` skill. It is required because this workflow depends on Leonardo's Brave profile and installed 1Password extension.
2. Add Browser Pi's Node modules directory to `node_repl`:

   ```text
   /Users/leonardo.gavaudan/.pi/agent/skills/browser-pi/node_modules
   ```

3. Import `scripts/login.mjs` through `node_repl`, then call:

   ```js
   var localLogin = await import(
     'file:///Users/leonardo.gavaudan/.config/codex/skills/algolia-local-login/scripts/login.mjs'
   );
   await localLogin.loginToAlgoliaLocal({
     port: 8367,
     applicationId: 'F4T6CUV2AH',
   });
   ```

   Override `targetPath` when a different authenticated route is needed.

The helper first reuses an existing localhost session. If login is required, it creates a hidden, stable 1Password extension context through CDP, verifies that this Browser Pi profile uses an independent extension lock, reads the Keychain-protected unlock secret, selects the unlocked 1Password account explicitly, retrieves the beta-dashboard login item inside the browser process, submits the local login form, and closes the temporary context. It also initializes a blocked local Hotjar stub and a `crypto.randomUUID` fallback so the authenticated page does not crash when beta's runtime globals or secure-context APIs are absent. It returns only safe status metadata and reports the exact failed stage.

Browser Pi intentionally keeps its own 1Password extension state when refreshing the rest of the profile from Brave. In the Browser Pi profile only, keep **Integrate this extension with the 1Password desktop app** disabled. This prevents an automation run from handing unlock to the native app and displaying a password dialog. It does not change Leonardo's normal Brave profile.

## Guardrails

- Never print, return, paste, or inspect credential field values.
- Never put the 1Password account password or Dashboard password in shell arguments, source files, PRs, or logs.
- Keep the Keychain item account fixed to `leonardo.gavaudan@algolia.com` unless Leonardo explicitly changes it.
- Keep the user-facing preview on `localhost`. Do not switch to the Mac's private IPv4 address unless Leonardo explicitly asks.
- Do not enable desktop-app integration in the Browser Pi profile. The helper stops with an actionable configuration error before attempting an unlock if shared lock state is enabled.
- Do not open the 1Password popup page for automation. Popup targets are transient and can close while an extension message is in flight. The helper's hidden inline context is intentional.
- If the helper reports that the Keychain item is missing, ask Leonardo to recreate that Keychain entry. Do not ask him to send a password in chat.
- If an old interactive 1Password unlock dialog is already visible, cancel it before retrying with the helper. A successful helper run should not create that dialog.
