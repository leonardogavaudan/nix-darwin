---
name: browser-pi
description: Interactive browser automation via Chrome DevTools Protocol. Use when you need to interact with web pages, test frontends, or when user interaction with a visible browser is required.
---

# Browser Tools

Chrome DevTools Protocol tools for agent-assisted web automation. In this setup they connect to Brave running on `:9222` with remote debugging enabled.

## Setup

Run once before first use:

```bash
cd {baseDir}/browser-pi
npm install
```

## Start Browser

```bash
{baseDir}/browser-start.js               # Start Brave (default, background)
{baseDir}/browser-start.js --profile     # Start Brave with Brave profile copy (background)
{baseDir}/browser-start.js --focus       # Start and bring browser to foreground
```

Launches **Brave** with remote debugging on `:9222` (Brave-only workflow).
Use `--profile` to preserve authentication state from the Brave profile.
By default this starts in the background to avoid stealing focus. Pass `--focus` when foreground is needed.

## List Tabs

```bash
{baseDir}/browser-tabs.js
```

Shows tab indices, titles, and URLs. Use the index with `--tab` in other commands.

## Navigate

```bash
{baseDir}/browser-nav.js https://example.com
{baseDir}/browser-nav.js https://example.com --new
{baseDir}/browser-nav.js https://example.com --tab 2
{baseDir}/browser-nav.js https://example.com --url-match circleci
{baseDir}/browser-nav.js https://example.com --focus
```

Navigate to URLs.
- Default behavior does **not** bring the tab/browser to front (prevents focus stealing)
- `--new` without `--focus`: opens a **background tab** via CDP (no app activation)
- `--focus`: bring the target tab to front when needed for interactive tasks
- `--tab <index>`: target a specific tab index
- `--url-match <substring>`: target the last tab whose URL contains the substring

## Evaluate JavaScript

```bash
{baseDir}/browser-eval.js 'document.title'
{baseDir}/browser-eval.js --tab 2 'document.title'
{baseDir}/browser-eval.js --url-match circleci 'location.href'
{baseDir}/browser-eval.js --focus 'document.querySelectorAll("a").length'
```

Execute JavaScript in a target tab. Code runs in async context. Use this to extract data, inspect page state, or perform DOM operations programmatically.
Default behavior does **not** bring the tab/browser to front; pass `--focus` when you explicitly want that.

## Screenshot

```bash
{baseDir}/browser-screenshot.js
```

Capture current viewport and return temporary file path.

**Important:** this captures the last active page viewport, not a semantic "diagram element". If the user reports clipping, it is usually one of:
- wrong tab selected,
- modal overlay still open,
- content itself clipped by app camera/frame,
- viewport too small for full content.

For reliable captures:
1. Run `browser-tabs.js` first and verify target tab index.
2. Bring target tab to front if needed.
3. For large diagrams/canvases, prefer element-level screenshots (`svg.screenshot(...)`) via a small Puppeteer script instead of viewport screenshots.

Excalidraw-specific rule:
- Prefer Pi local preview URLs (`http://localhost:8787/latest.html` or `.../<checkpointId>.html`) over `excalidraw.com/#json=...` for screenshots.
- This avoids the "Load from link / Replace my content" modal that appears on Excalidraw share imports.

## Pick Elements

```bash
{baseDir}/browser-pick.js "Click the submit button"
```

**IMPORTANT**: Use this tool when the user wants to select specific DOM elements on the page. This launches an interactive picker that lets the user click elements to select them. The user can select multiple elements (Cmd/Ctrl+Click) and press Enter when done. The tool returns CSS selectors for the selected elements.

Common use cases:
- User says "I want to click that button" → Use this tool to let them select it
- User says "extract data from these items" → Use this tool to let them select the elements
- When you need specific selectors but the page structure is complex or ambiguous

## Cookies

```bash
{baseDir}/browser-cookies.js
```

Display all cookies for the current tab including domain, path, httpOnly, and secure flags. Use this to debug authentication issues or inspect session state.

## Extract Page Content

```bash
{baseDir}/browser-content.js https://example.com
```

Navigate to a URL and extract readable content as markdown. Uses Mozilla Readability for article extraction and Turndown for HTML-to-markdown conversion. Works on pages with JavaScript content (waits for page to load).

## When to Use

- Testing frontend code in a real browser
- Interacting with pages that require JavaScript
- When user needs to visually see or interact with a page
- Debugging authentication or session issues
- Scraping dynamic content that requires JS execution

---

## Efficiency Guide

### DOM Inspection Over Screenshots

**Don't** take screenshots to see page state. **Do** parse the DOM directly:

```javascript
// Get page structure
document.body.innerHTML.slice(0, 5000)

// Find interactive elements
Array.from(document.querySelectorAll('button, input, [role="button"]')).map(e => ({
  id: e.id,
  text: e.textContent.trim(),
  class: e.className
}))
```

### Complex Scripts in Single Calls

Wrap everything in an IIFE to run multi-statement code:

```javascript
(function() {
  // Multiple operations
  const data = document.querySelector('#target').textContent;
  const buttons = document.querySelectorAll('button');
  
  // Interactions
  buttons[0].click();
  
  // Return results
  return JSON.stringify({ data, buttonCount: buttons.length });
})()
```

### Batch Interactions

**Don't** make separate calls for each click. **Do** batch them:

```javascript
(function() {
  const actions = ["btn1", "btn2", "btn3"];
  actions.forEach(id => document.getElementById(id).click());
  return "Done";
})()
```

### Typing/Input Sequences

```javascript
(function() {
  const text = "HELLO";
  for (const char of text) {
    document.getElementById("key-" + char).click();
  }
  document.getElementById("submit").click();
  return "Submitted: " + text;
})()
```

### Reading App/Game State

Extract structured state in one call:

```javascript
(function() {
  const state = {
    score: document.querySelector('.score')?.textContent,
    status: document.querySelector('.status')?.className,
    items: Array.from(document.querySelectorAll('.item')).map(el => ({
      text: el.textContent,
      active: el.classList.contains('active')
    }))
  };
  return JSON.stringify(state, null, 2);
})()
```

### Waiting for Updates

If DOM updates after actions, add a small delay with bash:

```bash
sleep 0.5 && {baseDir}/browser-eval.js '...'
```

### Investigate Before Interacting

Always start by understanding the page structure:

```javascript
(function() {
  return {
    title: document.title,
    forms: document.forms.length,
    buttons: document.querySelectorAll('button').length,
    inputs: document.querySelectorAll('input').length,
    mainContent: document.body.innerHTML.slice(0, 3000)
  };
})()
```

Then target specific elements based on what you find.
