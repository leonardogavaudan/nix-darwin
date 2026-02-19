#!/usr/bin/env node

import puppeteer from "puppeteer-core";

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
	if (pages.length === 0) {
		console.log("No tabs found.");
		process.exit(0);
	}

	const activeIndex = pages.length - 1;
	for (let i = 0; i < pages.length; i++) {
		const page = pages[i];
		const marker = i === activeIndex ? "*" : " ";
		let title = "";
		try {
			title = (await page.title()) || "(no title)";
		} catch {
			title = "(title unavailable)";
		}
		const url = page.url() || "about:blank";
		console.log(`[${i}]${marker} ${title}`);
		console.log(`    ${url}`);
	}
	console.log("\n* = tab used by browser-nav.js/browser-eval.js when no --tab/--url-match is set");
} finally {
	await b.disconnect();
}
