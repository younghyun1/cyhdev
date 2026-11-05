import { onSettled, createEffect, untrack } from "solid-js";
import type { JSX } from "@solidjs/web";
import Editor from "@toast-ui/editor";
import type { EditorOptions } from "@toast-ui/editor";
import "@toast-ui/editor/dist/toastui-editor.css";
import "@toast-ui/editor/dist/theme/toastui-editor-dark.css";
import { theme } from "../state/theme";
import { photographyApi } from "../services/all_api";
import { t } from "../state/i18n";

interface MarkdownEditorProps {
  value: string;
  onChange: (val: string) => void;
  options?: Partial<EditorOptions> & { minHeight?: string };
  class?: string;
}

/** Letters of Arabic, Hebrew, Syriac, Thaana, N'Ko, and their presentation forms. */
const RTL_LETTER = /[\u0590-\u08ff\ufb1d-\ufdff\ufe70-\ufefc]/;
const ANY_LETTER = /\p{L}/u;

/** Direction of a block's own text by its first letter, or null without letters. */
function firstLetterDirection(text: string): "rtl" | "ltr" | null {
  for (const ch of text) {
    if (ANY_LETTER.test(ch)) return RTL_LETTER.test(ch) ? "rtl" : "ltr";
  }
  return null;
}

/** Edited text is managed by ProseMirror, which drops attributes it does not know. */
const EDITING_ROOT_SELECTOR = ".ProseMirror";
/** The rendered preview is plain HTML, so each block can carry its own direction. */
const PREVIEW_BLOCK_SELECTOR =
  ".toastui-editor-md-preview .toastui-editor-contents :is(p, h1, h2, h3, h4, h5, h6)";
const PREVIEW_CONTAINER_SELECTOR =
  ".toastui-editor-md-preview .toastui-editor-contents :is(ul, ol, li, table, th, td, blockquote)";

/**
 * Directions for mixed-script text. The editing surface takes its direction
 * from its first letter, since ProseMirror removes attributes from the lines it
 * manages, and blog.css orders each line by its own script. Each text block of the preview takes its own direction; a block that
 * holds other blocks, such as a list or an item with paragraphs, takes it from
 * its text explicitly, because `auto` ignores children that carry a direction
 * of their own.
 */
function markDirections(root: HTMLElement): void {
  root.querySelectorAll(EDITING_ROOT_SELECTOR).forEach((surface) => {
    if (surface.getAttribute("dir") !== "auto") surface.setAttribute("dir", "auto");
  });
  root.querySelectorAll(PREVIEW_BLOCK_SELECTOR).forEach((block) => {
    if (block.getAttribute("dir") !== "auto") block.setAttribute("dir", "auto");
  });
  root.querySelectorAll(PREVIEW_CONTAINER_SELECTOR).forEach((container) => {
    const direction = firstLetterDirection(container.textContent ?? "");
    if (direction && container.getAttribute("dir") !== direction) {
      container.setAttribute("dir", direction);
    }
  });
}

export default function MarkdownEditor(
  props: MarkdownEditorProps,
): JSX.Element {
  let containerRef: HTMLDivElement | undefined;
  let editor: Editor | undefined;

  onSettled(() => {
    const { minHeight, hooks, ...restOptions } = props.options ?? {};
    const addImageBlobHook =
      hooks?.addImageBlobHook ??
      (async (
        blob: Blob | File,
        callback: (url: string, alt?: string) => void,
      ) => {
        try {
          const fileName = blob instanceof File ? blob.name : "post-image";
          const formData = new FormData();
          formData.append("file", blob, fileName);
          formData.append("context", "post");
          formData.append("comments", fileName);
          formData.append("lat", "0");
          formData.append("lon", "0");

          const resp = await photographyApi.uploadPhotograph(formData);
          if (!resp?.success || !resp.data?.photograph_link) {
            throw new Error("Upload failed");
          }
          callback(resp.data.photograph_link, fileName);
        } catch (err) {
          console.error("Image upload failed:", err);
          alert(t("markdown.image_upload_failed"));
        }
      });
    editor = new Editor({
      el: containerRef!,
      height: restOptions.height ?? minHeight ?? "100%",
      initialEditType: "markdown",
      previewStyle: "vertical",
      usageStatistics: false,
      initialValue: props.value ?? "",
      hooks: {
        ...hooks,
        addImageBlobHook,
      },
      ...restOptions,
    });
    editor.on("change", () => {
      if (!editor) return;
      const onEditorChange = untrack(() => props.onChange);
      onEditorChange(editor.getMarkdown());
    });

    // Edits rebuild lines and preview blocks, so marking follows each change,
    // coalesced to one pass per frame.
    let marking = 0;
    const observer = new MutationObserver(() => {
      if (marking) return;
      marking = requestAnimationFrame(() => {
        marking = 0;
        if (containerRef) markDirections(containerRef);
      });
    });
    observer.observe(containerRef!, { childList: true, subtree: true, characterData: true });
    markDirections(containerRef!);

    return () => {
      observer.disconnect();
      if (marking) cancelAnimationFrame(marking);
      editor?.destroy();
      editor = undefined;
    };
  });

  createEffect(
    () => theme() === "dark",
    (dark) => {
      if (containerRef) {
        containerRef.classList.toggle("toastui-editor-dark", dark);
      }
    },
  );

  createEffect(
    () => props.value ?? "",
    (next) => {
      if (!editor) return;
      if (next !== editor.getMarkdown()) {
        editor.setMarkdown(next, false);
      }
    },
  );

  // The editor chrome stays left to right; blog.css lets each paragraph of the
  // text itself take the direction of its own script.
  return <div ref={containerRef} dir="ltr" class={props.class ?? "w-full h-full"} />;
}
