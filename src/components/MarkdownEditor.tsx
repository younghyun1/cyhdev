import { onCleanup, onMount, createEffect } from "solid-js";
import type { JSX } from "solid-js";
import Editor from "@toast-ui/editor";
import type { EditorOptions } from "@toast-ui/editor";
import "@toast-ui/editor/dist/toastui-editor.css";
import "@toast-ui/editor/dist/theme/toastui-editor-dark.css";
import { theme } from "../state/theme";

interface MarkdownEditorProps {
  value: string;
  onChange: (val: string) => void;
  options?: Partial<EditorOptions> & { minHeight?: string };
  class?: string;
}

export default function MarkdownEditor(
  props: MarkdownEditorProps,
): JSX.Element {
  let containerRef: HTMLDivElement | undefined;
  let editor: Editor | undefined;

  onMount(() => {
    const { minHeight, ...restOptions } = props.options ?? {};
    editor = new Editor({
      el: containerRef!,
      height: restOptions.height ?? minHeight ?? "100%",
      initialEditType: "markdown",
      previewStyle: "vertical",
      usageStatistics: false,
      initialValue: props.value ?? "",
      ...restOptions,
    });
    editor.on("change", () => {
      props.onChange(editor!.getMarkdown());
    });
  });

  onCleanup(() => {
    editor?.destroy();
    editor = undefined;
  });

  createEffect(() => {
    if (containerRef) {
      containerRef.classList.toggle("toastui-editor-dark", theme() === "dark");
    }
  });

  createEffect(() => {
    if (!editor) return;
    const next = props.value ?? "";
    if (next !== editor.getMarkdown()) {
      editor.setMarkdown(next, false);
    }
  });

  return <div ref={containerRef} class={props.class ?? "w-full h-full"} />;
}
