import fs from "node:fs";

// OMP's compiled binary currently fails to resolve bare npm specifiers from flat
// extension files under ~/.omp/agent/extensions. Resolve dependencies explicitly
// from the mirrored sibling node_modules directory instead.
//
// Keep the ASCII/text fallback dependency-free. Nested bare imports inside
// beautiful-mermaid's bundle (for example `entities`) are not resolved reliably by
// the live OMP runtime from this extension context.
const typeboxUrl = new URL("./node_modules/@sinclair/typebox/build/esm/index.mjs", import.meta.url).href;
const mermaidIsomorphicUrl = new URL(
  "./node_modules/mermaid-isomorphic/dist/mermaid-isomorphic.js",
  import.meta.url,
).href;

let renderMermaidSchema: any = null;
let renderMermaidAsciiSafe: ((source: string) => string | null | undefined) | null = null;
let createMermaidRenderer: any = null;

const DEFAULT_BROWSER_PATHS = [
  process.env.OMP_MERMAID_BROWSER_PATH,
  process.env.MERMAID_BROWSER_PATH,
  "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Chromium.app/Contents/MacOS/Chromium",
  "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
].filter((value): value is string => Boolean(value));

let renderer: any = null;
let rendererBrowserPath: string | null = null;

function renderMermaidTextFallback(source: string): string | null {
  const normalized = source.replace(/\r\n/g, "\n").trim();
  if (!normalized) return null;

  return [
    "[Mermaid diagram fallback: raw source]",
    "",
    ...normalized.split("\n").map(line => `  ${line}`),
  ].join("\n");
}

async function ensureSchema() {
  if (!renderMermaidSchema) {
    const { Type } = await import(typeboxUrl);
    renderMermaidSchema = Type.Object({
      mermaid: Type.String({ description: "Mermaid graph source text" }),
      config: Type.Optional(
        Type.Object({
          useAscii: Type.Optional(Type.Boolean()),
          paddingX: Type.Optional(Type.Number()),
          paddingY: Type.Optional(Type.Number()),
          boxBorderPadding: Type.Optional(Type.Number()),
        }),
      ),
    });
  }

  return renderMermaidSchema;
}

async function ensureRenderDependencies() {
  if (!renderMermaidAsciiSafe || !createMermaidRenderer) {
    const mermaidIsomorphic = await import(mermaidIsomorphicUrl);

    renderMermaidAsciiSafe = renderMermaidTextFallback;
    createMermaidRenderer = mermaidIsomorphic.createMermaidRenderer;
  }

  return { renderMermaidAsciiSafe, createMermaidRenderer };
}


function findBrowserPath(): string | null {
  for (const candidate of DEFAULT_BROWSER_PATHS) {
    if (candidate && fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

function getRenderer() {
  const browserPath = findBrowserPath();
  if (!browserPath) {
    return { renderer: null, browserPath: null };
  }

  if (!renderer || rendererBrowserPath !== browserPath) {
    if (!createMermaidRenderer) {
      throw new Error("Mermaid renderer dependency was not initialized.");
    }

    renderer = createMermaidRenderer({
      launchOptions: {
        executablePath: browserPath,
        headless: true,
        args: ["--disable-gpu", "--no-sandbox", "--disable-dev-shm-usage"],
      },
    });
    rendererBrowserPath = browserPath;
  }

  return { renderer, browserPath };
}

function normalizeError(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }
  return String(error);
}

export default async function (pi: any) {
  const parameters = await ensureSchema();

  pi.registerTool({
    name: "render_mermaid",
    label: "RenderMermaid",
    description:
      "Render Mermaid diagrams as PNG images in OMP when possible. Falls back to ASCII text if image rendering is unavailable or fails.",
    parameters,
    promptSnippet: "Render Mermaid diagrams as terminal-friendly PNG images; use for diagrams that are hard to read as raw fenced Mermaid.",
    promptGuidelines: [
      "Prefer this tool over raw fenced Mermaid when the user wants the diagram rendered in the terminal.",
      "Use config.useAscii=true only when the user explicitly wants plain text output.",
    ],
    async execute(_toolCallId: string, params: { mermaid: string; config?: { useAscii?: boolean } }) {
      const { renderMermaidAsciiSafe } = await ensureRenderDependencies();
      const ascii = renderMermaidAsciiSafe?.(params.mermaid) ?? null;
      const wantsAscii = params.config?.useAscii === true;

      if (!wantsAscii) {
        const { renderer, browserPath } = getRenderer();
        if (renderer && browserPath) {
          try {
            const [result] = await renderer([params.mermaid], {
              screenshot: true,
              mermaidOptions: {
                theme: "default",
                securityLevel: "strict",
                fontFamily: "Arial, sans-serif",
                flowchart: { useMaxWidth: false },
              },
              containerStyle: {
                maxHeight: "none",
                maxWidth: "none",
                opacity: "1",
                overflow: "visible",
                padding: "24px",
                background: "white",
              },
            });

            if (result?.status === "fulfilled" && result.value?.screenshot) {
              const png = Buffer.from(result.value.screenshot).toString("base64");
              const width = result.value.width ?? undefined;
              const height = result.value.height ?? undefined;
              return {
                content: [
                  {
                    type: "text",
                    text:
                      width && height
                        ? `Rendered Mermaid diagram as PNG via ${browserPath} (${width}x${height}).`
                        : `Rendered Mermaid diagram as PNG via ${browserPath}.`,
                  },
                  { type: "image", data: png, mimeType: "image/png" },
                ],
                details: {
                  mode: "image",
                  browserPath,
                  width,
                  height,
                },
              };
            }

            const reason =
              result?.status === "rejected" ? normalizeError(result.reason) : "Renderer returned no PNG output.";

            if (ascii) {
              return {
                content: [
                  {
                    type: "text",
                    text: `PNG render failed (${reason}). Showing ASCII fallback instead.\n\n${ascii}`,
                  },
                ],
                details: {
                  mode: "ascii-fallback",
                  browserPath,
                  error: reason,
                },
              };
            }

            return {
              content: [{ type: "text", text: `Mermaid render failed: ${reason}` }],
              details: {
                mode: "error",
                browserPath,
                error: reason,
              },
            };
          } catch (error) {
            const reason = normalizeError(error);
            if (ascii) {
              return {
                content: [
                  {
                    type: "text",
                    text: `PNG render failed (${reason}). Showing ASCII fallback instead.\n\n${ascii}`,
                  },
                ],
                details: {
                  mode: "ascii-fallback",
                  browserPath,
                  error: reason,
                },
              };
            }

            return {
              content: [{ type: "text", text: `Mermaid render failed: ${reason}` }],
              details: {
                mode: "error",
                browserPath,
                error: reason,
              },
            };
          }
        }
      }

      if (ascii) {
        const browserHint = wantsAscii
          ? "ASCII mode was requested explicitly."
          : "No compatible browser was found for PNG rendering. Set OMP_MERMAID_BROWSER_PATH if needed.";
        return {
          content: [{ type: "text", text: `${browserHint}\n\n${ascii}` }],
          details: {
            mode: "ascii",
            browserPath: null,
          },
        };
      }

      return {
        content: [
          {
            type: "text",
            text: wantsAscii
              ? "ASCII Mermaid rendering failed."
              : "Mermaid rendering failed and no ASCII fallback was available.",
          },
        ],
        details: {
          mode: "error",
          browserPath: null,
        },
      };
    },
  });
}
