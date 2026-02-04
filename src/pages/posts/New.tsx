import { createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { blogApi } from "../../services/all_api";
import MarkdownEditor from "../../components/MarkdownEditor";
import { pageStyles } from "../../styles/pageStyles";
export default function NewPostPage() {
  const [title, setTitle] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [body, setBody] = createSignal("");
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
        post_is_published: true,
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
            <div class="w-full h-[28rem] min-w-0">
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
            <button
              type="submit"
              disabled={isSubmitting()}
              class={pageStyles.buttonPrimary}
            >
              {isSubmitting() ? "Publishing..." : "Publish"}
            </button>
          </form>
        </div>
        {/* Optionally, place a sidebar here if you want Reddit-style right column */}
      </div>
    </main>
  );
}
