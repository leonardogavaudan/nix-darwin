import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));

const loggerPath = path.join(here, "node_modules", "@oh-my-pi", "pi-utils", "src", "logger.ts");
const mermaidAsciiPath = path.join(here, "node_modules", "@oh-my-pi", "pi-utils", "src", "mermaid-ascii.ts");
const mermaidAsciiPatchPath = path.join(here, "patched-pi-utils-mermaid-ascii.ts");
const extensionUiControllerPath = path.join(
  here,
  "node_modules",
  "@oh-my-pi",
  "pi-coding-agent",
  "src",
  "modes",
  "controllers",
  "extension-ui-controller.ts",
);
const interactiveModePath = path.join(
  here,
  "node_modules",
  "@oh-my-pi",
  "pi-coding-agent",
  "src",
  "modes",
  "interactive-mode.ts",
);

const brokenLoggerImport = 'import { RingBuffer } from "@oh-my-pi/pi-utils/ring";';
const fixedLoggerImport = 'import { RingBuffer } from "./ring.ts";';
const interactiveReloadBefore = `			reload: async () => {
				await this.ctx.session.reload();
				this.ctx.chatContainer.clear();
				this.ctx.renderInitialMessages();
				await this.ctx.reloadTodos();
				this.ctx.showStatus("Reloaded session");
			},`;
const interactiveReloadWorkaround = `			reload: async () => {
				if (this.ctx.loadingAnimation) {
					this.ctx.loadingAnimation.stop();
					this.ctx.loadingAnimation = undefined;
				}
				this.ctx.statusContainer.clear();
				this.ctx.pendingMessagesContainer.clear();
				this.ctx.compactionQueuedMessages = [];
				this.ctx.streamingComponent = undefined;
				this.ctx.streamingMessage = undefined;
				this.ctx.pendingTools.clear();
				await this.ctx.session.reload();
				this.ctx.chatContainer.clear();
				this.ctx.renderInitialMessages();
				await this.ctx.reloadTodos();
				this.ctx.showStatus("Reloaded session");
			},`;
const backgroundReloadBefore = `			reload: async () => {
				if (this.ctx.isBackgrounded) {
					return;
				}
				await this.ctx.session.reload();
				this.ctx.chatContainer.clear();
				this.ctx.renderInitialMessages();
				await this.ctx.reloadTodos();
				this.ctx.showStatus("Reloaded session");
			},`;
const backgroundReloadWorkaround = `			reload: async () => {
				if (this.ctx.isBackgrounded) {
					return;
				}
				if (this.ctx.loadingAnimation) {
					this.ctx.loadingAnimation.stop();
					this.ctx.loadingAnimation = undefined;
				}
				this.ctx.statusContainer.clear();
				this.ctx.pendingMessagesContainer.clear();
				this.ctx.compactionQueuedMessages = [];
				this.ctx.streamingComponent = undefined;
				this.ctx.streamingMessage = undefined;
				this.ctx.pendingTools.clear();
				await this.ctx.session.reload();
				this.ctx.chatContainer.clear();
				this.ctx.renderInitialMessages();
				await this.ctx.reloadTodos();
				this.ctx.showStatus("Reloaded session");
			},`;
const finishPendingSubmissionBefore = `	finishPendingSubmission(input: SubmittedUserInput): void {
		if (this.#pendingSubmittedInput === input) {
			this.#pendingSubmittedInput = undefined;
		}
	}`;
const finishPendingSubmissionAfter = `	finishPendingSubmission(input: SubmittedUserInput): void {
		if (this.#pendingSubmittedInput !== input) {
			return;
		}
		this.#pendingSubmittedInput = undefined;
		this.optimisticUserMessageSignature = undefined;
		this.#pendingWorkingMessage = undefined;
		if (this.loadingAnimation) {
			this.loadingAnimation.stop();
			this.loadingAnimation = undefined;
			this.statusContainer.clear();
		}
	}`;

function replaceSnippet(filePath, before, after) {
  if (!fs.existsSync(filePath)) return false;

  const current = fs.readFileSync(filePath, "utf8");
  if (current.includes(after)) return false;
  if (!current.includes(before)) {
    console.warn(`Did not find expected snippet in ${filePath}`);
    return false;
  }

  fs.writeFileSync(filePath, current.replace(before, after));
  console.log(`Patched ${filePath}`);
  return true;
}

function patchLogger() {
  replaceSnippet(loggerPath, brokenLoggerImport, fixedLoggerImport);
}

function patchMermaidAscii() {
  if (!fs.existsSync(mermaidAsciiPath) || !fs.existsSync(mermaidAsciiPatchPath)) return;

  const desired = fs.readFileSync(mermaidAsciiPatchPath, "utf8");
  const current = fs.readFileSync(mermaidAsciiPath, "utf8");
  if (current === desired) return;

  fs.writeFileSync(mermaidAsciiPath, desired);
  console.log(`Patched ${mermaidAsciiPath}`);
}

function normalizeExtensionUiControllerReload() {
  replaceSnippet(extensionUiControllerPath, interactiveReloadWorkaround, interactiveReloadBefore);
  replaceSnippet(extensionUiControllerPath, backgroundReloadWorkaround, backgroundReloadBefore);
}

function patchInteractiveModePendingSubmission() {
  replaceSnippet(interactiveModePath, finishPendingSubmissionBefore, finishPendingSubmissionAfter);
}

patchLogger();
patchMermaidAscii();
normalizeExtensionUiControllerReload();
patchInteractiveModePendingSubmission();
