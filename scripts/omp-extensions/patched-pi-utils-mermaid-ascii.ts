export interface MermaidAsciiRenderOptions {
  useAscii?: boolean;
  paddingX?: number;
  paddingY?: number;
  boxBorderPadding?: number;
}

function renderMermaidText(source: string): string | null {
  const normalized = source.replace(/\r\n/g, "\n").trim();
  if (!normalized) return null;

  return [
    "[Mermaid diagram fallback: raw source]",
    "",
    ...normalized.split("\n").map(line => `  ${line}`),
  ].join("\n");
}

export function renderMermaidAscii(source: string, _options?: MermaidAsciiRenderOptions): string {
  return renderMermaidText(source) ?? "";
}

export function renderMermaidAsciiSafe(source: string, _options?: MermaidAsciiRenderOptions): string | null {
  return renderMermaidText(source);
}

/**
 * Extract mermaid code blocks from markdown text.
 */
export function extractMermaidBlocks(markdown: string): { source: string; hash: bigint }[] {
  const blocks: { source: string; hash: bigint }[] = [];
  const regex = /```mermaid\s*\n([\s\S]*?)```/g;

  for (let match = regex.exec(markdown); match !== null; match = regex.exec(markdown)) {
    const source = match[1].trim();
    const hash = Bun.hash.xxHash64(source);
    blocks.push({ source, hash });
  }

  return blocks;
}
