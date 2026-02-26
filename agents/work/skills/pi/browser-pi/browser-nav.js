#!/usr/bin/env node

import puppeteer from "puppeteer-core";

function printUsage() {
	console.log(
		"Usage: browser-nav.js <url> [--new] [--reload] [--tab <index>] [--url-match <substring>] [--focus]",
	);
	console.log("\nExamples:");
	console.log("  browser-nav.js https://example.com                        # Navigate last tab (no focus)");
	console.log("  browser-nav.js https://example.com --new                  # Open in new tab (no focus)");
	console.log("  browser-nav.js https://example.com --tab 2                # Navigate tab index 2 (no focus)");
	console.log("  browser-nav.js https://example.com --url-match circleci   # Navigate last tab matching URL (no focus)");
	console.log("  browser-nav.js https://example.com --reload               # Navigate and force reload (no focus)");
	console.log("  browser-nav.js https://example.com --focus                # Navigate and bring tab to front");
}

function pickPage(pages, tabIndex, urlMatch) {
	if (pages.length === 0) {
		throw new Error("No tabs found in browser");
	}

	if (tabIndex !== null) {
		if (Number.isNaN(tabIndex) || tabIndex < 0 || tabIndex >= pages.length) {
			throw new Error(`Invalid --tab index ${tabIndex}. Available range: 0..${pages.length - 1}`);
		}
		return { page: pages[tabIndex], index: tabIndex };
	}

	if (urlMatch) {
		for (let i = pages.length - 1; i >= 0; i--) {
			if (pages[i].url().includes(urlMatch)) {
				return { page: pages[i], index: i };
			}
		}
		throw new Error(`No tab URL matched substring: ${urlMatch}`);
	}

	const index = pages.length - 1;
	return { page: pages[index], index };
}

async function openBackgroundTab(browser, url) {
	const session = await browser.target().createCDPSession();
	try {
		await session.send("Target.createTarget", { url, background: true });
	} finally {
		await session.detach().catch(() => {});
	}
}

const args = process.argv.slice(2);
let newTab = false;
let reload = false;
let focus = false;
let tabIndex = null;
let urlMatch = null;
let url = null;

for (let i = 0; i < args.length; i++) {
	const arg = args[i];

	if (arg === "--new") {
		newTab = true;
	} else if (arg === "--reload") {
		reload = true;
	} else if (arg === "--focus") {
		focus = true;
	} else if (arg === "--tab") {
		const value = args[i + 1];
		if (value === undefined) {
			console.error("✗ Missing value for --tab");
			printUsage();
			process.exit(1);
		}
		tabIndex = Number.parseInt(value, 10);
		i++;
	} else if (arg === "--url-match") {
		const value = args[i + 1];
		if (value === undefined) {
			console.error("✗ Missing value for --url-match");
			printUsage();
			process.exit(1);
		}
		urlMatch = value;
		i++;
	} else if (arg.startsWith("--")) {
		console.error(`✗ Unknown flag: ${arg}`);
		printUsage();
		process.exit(1);
	} else if (!url) {
		url = arg;
	} else {
		console.error(`✗ Unexpected extra argument: ${arg}`);
		printUsage();
		process.exit(1);
	}
}

if (!url) {
	printUsage();
	process.exit(1);
}

if (newTab && (tabIndex !== null || urlMatch)) {
	console.error("✗ --new cannot be combined with --tab or --url-match");
	process.exit(1);
}

const b = await Promise.race([
	puppeteer.connect({
		browserURL: "http://localhost:9222",
		defaultViewport: null,
	}),
	new Promise((_, reject) => setTimeout(() => reject(new Error("timeout")), 5000)),
]).catch((e) => {
	console.error("✗ Could not connect to browser:", e.message);
	console.error("  Run: browser-start.js");
	process.exit(1);
});

try {
	if (newTab) {
		if (focus) {
			const p = await b.newPage();
			await p.bringToFront();
			await p.goto(url, { waitUntil: "domcontentloaded" });
			console.log("✓ Opened:", url);
		} else {
			await openBackgroundTab(b, url);
			console.log("✓ Opened in background:", url);
		}
	} else {
		const pages = await b.pages();
		const { page, index } = pickPage(pages, tabIndex, urlMatch);
		if (focus) await page.bringToFront();
		await page.goto(url, { waitUntil: "domcontentloaded" });
		if (reload) {
			await page.reload({ waitUntil: "domcontentloaded" });
		}
		console.log(`✓ Navigated tab ${index} to: ${url}`);
	}
} finally {
	await b.disconnect();
}
