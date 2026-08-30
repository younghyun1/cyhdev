/// <reference types="vite/client" />

declare module "@toast-ui/editor" {
  import type { EditorOptions as _EditorOptions } from "@toast-ui/editor/types";
  export type EditorOptions = _EditorOptions;
  export default class Editor {
    constructor(options: Partial<EditorOptions> & { el: HTMLElement });
    getMarkdown(): string;
    setMarkdown(markdown: string, cursorToEnd?: boolean): void;
    on(event: string, callback: () => void): void;
    destroy(): void;
  }
}

declare module "turndown" {
  export default class TurndownService {
    constructor(options?: { codeBlockStyle?: string });
    use(plugin: unknown): this;
    turndown(html: string): string;
  }
}

declare module "turndown-plugin-gfm" {
  export const gfm: unknown;
}
