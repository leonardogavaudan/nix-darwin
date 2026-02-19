#!/usr/bin/env node

import { spawn, execSync } from "node:child_process";
import { existsSync } from "node:fs";
import puppeteer from "puppeteer-core";

const args = process.argv.slice(2);
const useProfile = args.includes("--profile");
const forceChrome = args.includes("--chrome");
const forceBrave = args.includes("--brave");

const invalidArg = args.find((arg) => !["--profile", "--brave", "--chrome"].includes(arg));
if (invalidArg) {
	console.log("Usage: browser-start.js [--profile] [--brave|--chrome]");
	console.log("\nOptions:");
	console.log("  --profile  Copy browser profile (cookies, logins)");
	console.log("  --brave    Force Brave (default)");
	console.log("  --chrome   Force Google Chrome");
	process.exit(1);
}

if (forceChrome && forceBrave) {
	console.error("✗ Use only one of --brave or --chrome");
	process.exit(1);
}

// Default browser is Brave unless explicitly overridden.
const useBrave = forceBrave || !forceChrome;

const browserName = useBrave ? "Brave" : "Chrome";
const browserBinary = useBrave
	? "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"
	: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const profileSource = useBrave
	? `${process.env.HOME}/Library/Application Support/BraveSoftware/Brave-Browser/`
	: `${process.env.HOME}/Library/Application Support/Google/Chrome/`;
const SCRAPING_DIR = `${process.env.HOME}/.cache/browser-pi`;

if (!existsSync(browserBinary)) {
	console.error(`✗ ${browserName} binary not found at: ${browserBinary}`);
	process.exit(1);
}

if (useProfile && !existsSync(profileSource)) {
	console.error(`✗ ${browserName} profile not found at: ${profileSource}`);
	process.exit(1);
}

// Check if already running on :9222
try {
	const browser = await puppeteer.connect({
		browserURL: "http://localhost:9222",
		defaultViewport: null,
	});
	await browser.disconnect();
	console.log("✓ Browser already running on :9222");
	process.exit(0);
} catch {}

// Setup profile directory
execSync(`mkdir -p "${SCRAPING_DIR}"`, { stdio: "ignore" });

// Remove SingletonLock to allow new instance
try {
	execSync(
		`rm -f "${SCRAPING_DIR}/SingletonLock" "${SCRAPING_DIR}/SingletonSocket" "${SCRAPING_DIR}/SingletonCookie"`,
		{ stdio: "ignore" },
	);
} catch {}

if (useProfile) {
	console.log(`Syncing ${browserName} profile...`);
	execSync(
		`rsync -a --delete \
			--exclude='SingletonLock' \
			--exclude='SingletonSocket' \
			--exclude='SingletonCookie' \
			--exclude='*/Sessions/*' \
			--exclude='*/Current Session' \
			--exclude='*/Current Tabs' \
			--exclude='*/Last Session' \
			--exclude='*/Last Tabs' \
			"${profileSource}" "${SCRAPING_DIR}/"`,
		{ stdio: "pipe" },
	);
}

// Start browser with flags to force new instance
spawn(
	browserBinary,
	[
		"--remote-debugging-port=9222",
		`--user-data-dir=${SCRAPING_DIR}`,
		"--no-first-run",
		"--no-default-browser-check",
	],
	{ detached: true, stdio: "ignore" },
).unref();

// Wait for browser to be ready
let connected = false;
for (let i = 0; i < 30; i++) {
	try {
		const browser = await puppeteer.connect({
			browserURL: "http://localhost:9222",
			defaultViewport: null,
		});
		await browser.disconnect();
		connected = true;
		break;
	} catch {
		await new Promise((r) => setTimeout(r, 500));
	}
}

if (!connected) {
	console.error(`✗ Failed to connect to ${browserName}`);
	process.exit(1);
}

console.log(`✓ ${browserName} started on :9222${useProfile ? " with profile" : ""}`);
