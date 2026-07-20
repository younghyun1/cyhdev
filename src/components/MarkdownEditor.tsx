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

    return () => {
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

  return <div ref={containerRef} class={props.class ?? "w-full h-full"} />;
}
