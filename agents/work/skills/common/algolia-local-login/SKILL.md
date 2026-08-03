---
name: algolia-local-login
description: Sign into a local AlgoliaWeb frontend backed by beta-dashboard using Leonardo's Brave Browser Pi profile, the 1Password browser extension, and a macOS Keychain-protected unlock secret. Use when starting or visually testing AlgoliaWeb locally without a local Rails backend, especially when localhost redirects, 1Password unlocks, or repeated login steps are slowing down UI QA.
---

# Algolia Local Login

Run a frontend-only AlgoliaWeb preview against the beta backend, then sign in without exposing credentials in commands or tool output.

## Start the frontend

From the worktree's `_client` directory, expose Vite on the Mac's local network interface because Brave's Browser Pi profile can reject loopback connections intermittently:

```bash
DASHBOARD_WEB_BACKEND_HOST=https://beta-dashboard.algolia.com yarn vite --host 0.0.0.0 --port <port>
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

The helper reads the 1Password unlock secret from macOS Keychain service `algolia-1password-unlock`, unlocks the extension if necessary, retrieves the beta-dashboard login item inside the browser process, submits the local login form, and navigates to the requested route. It also initializes a blocked local Hotjar stub and a `crypto.randomUUID` fallback so the authenticated page does not crash when beta's runtime globals or secure-context APIs are absent. It returns only safe status metadata.

## Guardrails

- Never print, return, paste, or inspect credential field values.
- Never put the 1Password account password or Dashboard password in shell arguments, source files, PRs, or logs.
- Keep the Keychain item account fixed to `leonardo.gavaudan@algolia.com` unless Leonardo explicitly changes it.
- Let the helper detect the Mac's private IPv4 address. The beta login response can redirect to `localhost`; the helper avoids depending on that redirect and opens the target route directly after the session cookie is set.
- If the helper reports that the Keychain item is missing, ask Leonardo to recreate that Keychain entry. Do not ask him to send a password in chat.
