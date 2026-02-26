#!/usr/bin/env node

import puppeteer from "puppeteer-core";

function printUsage() {
	console.log("Usage: browser-eval.js [--tab <index>] [--url-match <substring>] [--focus] '<code>'");
	console.log("\nExamples:");
	console.log('  browser-eval.js "document.title"                                  # No focus');
	console.log('  browser-eval.js --tab 2 "document.title"                          # No focus');
	console.log('  browser-eval.js --url-match circleci "location.href"              # No focus');
	console.log('  browser-eval.js --focus "document.querySelectorAll(\\"a\\").length"  # Bring tab to front');
}

function pickPage(pages, tabIndex, urlMatch) {
	if (pages.length === 0) {
		throw new Error("No tabs found in browser");
	}

	if (tabIndex !== null) {
		if (Number.isNaN(tabIndex) || tabIndex < 0 || tabIndex >= pages.length) {
			throw new Error(`Invalid --tab index ${tabIndex}. Available range: 0..${pages.length - 1}`);
		}
		return pages[tabIndex];
	}

	if (urlMatch) {
		for (let i = pages.length - 1; i >= 0; i--) {
			if (pages[i].url().includes(urlMatch)) return pages[i];
		}
		throw new Error(`No tab URL matched substring: ${urlMatch}`);
	}

	return pages.at(-1);
}

const args = process.argv.slice(2);
let tabIndex = null;
let urlMatch = null;
let focus = false;
const codeParts = [];

for (let i = 0; i < args.length; i++) {
	const arg = args[i];
	if (arg === "--tab") {
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
	} else if (arg === "--focus") {
		focus = true;
	} else {
		codeParts.push(arg);
	}
}

const code = codeParts.join(" ").trim();
if (!code) {
	printUsage();
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
	const pages = await b.pages();
	const p = pickPage(pages, tabIndex, urlMatch);
	if (!p) {
		console.error("✗ No active tab found");
		process.exit(1);
	}

	if (focus) await p.bringToFront();
	const result = await p.evaluate((c) => {
		const AsyncFunction = (async () => {}).constructor;
		return new AsyncFunction(`return (${c})`)();
	}, code);

	if (Array.isArray(result)) {
		for (let i = 0; i < result.length; i++) {
			if (i > 0) console.log("");
			for (const [key, value] of Object.entries(result[i])) {
				console.log(`${key}: ${value}`);
			}
		}
	} else if (typeof result === "object" && result !== null) {
		for (const [key, value] of Object.entries(result)) {
			console.log(`${key}: ${value}`);
		}
	} else {
		console.log(result);
	}
} finally {
	await b.disconnect();
}
