import { createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { blogApi } from "../../services/all_api";
import MarkdownEditor from "../../components/MarkdownEditor";
import { pageStyles } from "../../styles/pageStyles";
export default function NewPostPage() {
  const [title, setTitle] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [body, setBody] = createSignal("");
  const [isPublished, setIsPublished] = createSignal(true);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const navigate = useNavigate();

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setIsSubmitting(true);
    setError(null);

    const postTags = tags()
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

    try {
      const res = await blogApi.submitPost({
        post_title: title(),
        post_content: body(),
        post_tags: postTags,
        post_is_published: isPublished(),
      });
      if (res.success) {
        navigate(`/blog/${res.data.post_id}`, { replace: true });
      } else {
        setError("Failed to publish post.");
      }
    } catch (e: any) {
      setError(e?.message ?? "Failed to submit post.");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <main class={pageStyles.page}>
      <div class={`${pageStyles.pageInner} flex flex-row gap-8`}>
        <div class="flex-1">
          <h2 class={`${pageStyles.titleSm} mb-4`}>New Post</h2>
          <form onSubmit={handleSubmit} class="flex flex-col gap-4">
            <input
              type="text"
              placeholder="Title"
              value={title()}
              onInput={(e) => setTitle(e.currentTarget.value)}
              required
              class={pageStyles.input}
            />
            <input
              type="text"
              placeholder="Tags (comma separated)"
              value={tags()}
              onInput={(e) => setTags(e.currentTarget.value)}
              class={pageStyles.input}
            />
            <label class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-300">
              <input
                type="checkbox"
                checked={isPublished()}
                onChange={(e) => setIsPublished(e.currentTarget.checked)}
                class="h-4 w-4 rounded border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100"
              />
              Publish immediately
            </label>
            {!isPublished() && (
              <div class={pageStyles.muted}>
                This will be saved as a draft and only visible to superusers.
              </div>
            )}
            <div class="w-full h-112 min-w-0 relative">
              <label class="font-medium text-slate-700 dark:text-slate-200 mb-2 block">
                Content (Markdown)
              </label>
              <div class="h-full overflow-hidden">
                <MarkdownEditor
                  value={body()}
                  onChange={setBody}
                  options={{ minHeight: "100%" }}
                />
              </div>
            </div>
            {error() && <div class={pageStyles.alertError}>{error()}</div>}
            <div class="flex gap-4 relative z-10">
              <button
                type="submit"
                disabled={isSubmitting()}
                class={pageStyles.buttonPrimary}
              >
                {isSubmitting()
                  ? isPublished()
                    ? "Publishing..."
                    : "Saving..."
                  : isPublished()
                    ? "Publish"
                    : "Save Draft"}
              </button>
              <button
                type="button"
                onClick={() => navigate("/blog")}
                class={pageStyles.buttonSecondary}
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
        {/* Optionally, place a sidebar here if you want Reddit-style right column */}
      </div>
    </main>
  );
}
