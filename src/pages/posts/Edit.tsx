import { createSignal, onMount, Show } from "solid-js";
import { useNavigate, useParams } from "@solidjs/router";
import { blogApi } from "../../services/all_api";
import MarkdownEditor from "../../components/MarkdownEditor";
import { pageStyles } from "../../styles/pageStyles";
import TurndownService from "turndown";
import { gfm } from "turndown-plugin-gfm";

const htmlTagPattern = /<\/?[a-z][\s\S]*>/i;
const turndownService = new TurndownService({
  codeBlockStyle: "fenced",
});
turndownService.use(gfm);

const normalizeMarkdown = (content: string) => {
  if (!content) return "";
  if (!htmlTagPattern.test(content)) return content;
  try {
    return turndownService.turndown(content);
  } catch {
    return content;
  }
};

export default function EditPostPage() {
  const params = useParams();
  const navigate = useNavigate();

  const [title, setTitle] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [body, setBody] = createSignal("");
  const [isPublished, setIsPublished] = createSignal(true);
  const [initialPublished, setInitialPublished] = createSignal<boolean | null>(
    null,
  );
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  onMount(() => {
    void (async () => {
      const postId = params.post_id;
      if (!postId) {
        setError("No post ID provided");
        setIsLoading(false);
        return;
      }

      try {
        const res = await blogApi.readPost(postId);
        if (res.success && res.data) {
          const post = res.data.post;
          setTitle(post.post_title);
          setIsPublished(post.post_is_published);
          setInitialPublished(post.post_is_published);
          const markdownFromMetadata = post.post_metadata?.markdown_content;
          if (
            typeof markdownFromMetadata === "string" &&
            markdownFromMetadata.trim().length > 0
          ) {
            setBody(markdownFromMetadata);
          } else {
            setBody(normalizeMarkdown(post.post_content));
          }
          // Load tags from response
          if (res.data.post_tags && res.data.post_tags.length > 0) {
            setTags(res.data.post_tags.join(", "));
          }
        } else {
          setError("Failed to load post.");
        }
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : "Failed to load post.");
      } finally {
        setIsLoading(false);
      }
    })();
  });

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setIsSubmitting(true);
    setError(null);

    const postTags = tags()
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

    try {
      const res = await blogApi.updatePost(
        {
          post_title: title(),
          post_content: body(),
          post_tags: postTags,
          post_is_published: isPublished(),
        },
        params.post_id!,
      );
      if (res.success) {
        navigate(
          `/blog/${encodeURIComponent(res.data.post_slug || res.data.post_id)}`,
          {
            replace: true,
          },
        );
      } else {
        setError("Failed to update post.");
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to update post.");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <main class={pageStyles.page}>
      <div class={`${pageStyles.pageInner} flex flex-row gap-8`}>
        <div class="flex-1">
          <h2 class={`${pageStyles.titleSm} mb-4`}>Edit Post</h2>

          <Show when={isLoading()}>
            <div class={pageStyles.muted}>Loading post...</div>
          </Show>

          <Show when={!isLoading()}>
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
                Published
              </label>
              {!isPublished() && (
                <div class={pageStyles.muted}>
                  This will remain a draft and only visible to superusers.
                </div>
              )}
              <div class="w-full min-w-0 mb-8 relative z-0">
                <label class="font-medium text-slate-700 dark:text-slate-200 mb-2 block">
                  Content (Markdown)
                </label>
                <MarkdownEditor
                  value={body()}
                  onChange={setBody}
                  options={{ height: "28rem" }}
                />
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
                      ? "Updating..."
                      : "Saving..."
                    : isPublished() && initialPublished() === false
                      ? "Publish"
                      : isPublished()
                        ? "Update"
                        : "Save Draft"}
                </button>
                <button
                  type="button"
                  onClick={() => navigate(`/blog/${params.post_id}`)}
                  class={pageStyles.buttonSecondary}
                >
                  Cancel
                </button>
              </div>
            </form>
          </Show>
        </div>
      </div>
    </main>
  );
}
