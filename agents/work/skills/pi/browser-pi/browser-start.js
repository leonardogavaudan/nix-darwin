#!/usr/bin/env node

import { spawn, execSync } from "node:child_process";
import { existsSync } from "node:fs";
import puppeteer from "puppeteer-core";

const args = process.argv.slice(2);
const useProfile = args.includes("--profile");
const focus = args.includes("--focus");

const invalidArg = args.find((arg) => !["--profile", "--focus"].includes(arg));
if (invalidArg) {
	console.log("Usage: browser-start.js [--profile] [--focus]");
	console.log("\nOptions:");
	console.log("  --profile  Copy Brave profile (cookies, logins)");
	console.log("  --focus    Bring browser to foreground (default starts in background)");
	console.log("\nNote: this tool is Brave-only; Chrome flags are intentionally unsupported.");
	process.exit(1);
}

const browserName = "Brave";
const browserApp = "/Applications/Brave Browser.app";
const browserBinary = "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser";
const profileSource = `${process.env.HOME}/Library/Application Support/BraveSoftware/Brave-Browser/`;
const SCRAPING_DIR = `${process.env.HOME}/.cache/browser-pi`;
const ONE_PASSWORD_EXTENSION_ID = "aeblfdkhhhdcdjpifhhbdiojplfjncoa";

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
			--exclude='Default/Local Extension Settings/${ONE_PASSWORD_EXTENSION_ID}/' \
			--exclude='Default/IndexedDB/chrome-extension_${ONE_PASSWORD_EXTENSION_ID}_0.indexeddb.leveldb/' \
			"${profileSource}" "${SCRAPING_DIR}/"`,
		{ stdio: "pipe" },
	);
}

// Start browser with flags to force new instance.
// Use `open -g` by default to avoid stealing focus.
const openArgs = ["-n"];
if (!focus) openArgs.push("-g");
openArgs.push(
	"-a",
	browserApp,
	"--args",
	"--remote-debugging-port=9222",
	`--user-data-dir=${SCRAPING_DIR}`,
	"--no-first-run",
	"--no-default-browser-check",
);

spawn("open", openArgs, { detached: true, stdio: "ignore" }).unref();

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

console.log(
	`✓ ${browserName} started on :9222${useProfile ? " with profile" : ""}${focus ? " (focused)" : " (background)"}`,
);
