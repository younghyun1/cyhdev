import { createSignal, onSettled, Show } from "solid-js";
import { useNavigate, useParams } from "@solidjs/router";
import { blogApi } from "../../services/all_api";
import type { UpdatePostRequest } from "../../generated";
import MarkdownEditor from "../../components/MarkdownEditor";
import { pageStyles } from "../../styles/pageStyles";
import TurndownService from "turndown";
import { gfm } from "turndown-plugin-gfm";
import { t } from "../../state/i18n";

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

const markdownContentFromMetadata = (metadata: unknown): string | null => {
  if (
    typeof metadata !== "object" ||
    metadata === null ||
    !("markdown_content" in metadata)
  ) {
    return null;
  }
  const markdownContent = metadata.markdown_content;
  return typeof markdownContent === "string" ? markdownContent : null;
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
  // Canonical post UUID from the loaded post. The route param may be a slug
  // (read endpoint accepts both), but updatePost parses post_id as a UUID.
  const [canonicalPostId, setCanonicalPostId] = createSignal<string | null>(
    null,
  );

  onSettled(() => {
    void (async () => {
      const postId = params.post_id;
      if (!postId) {
        setError(t("blog.post.no_id"));
        setIsLoading(false);
        return;
      }

      try {
        const res = await blogApi.readPost(postId);
        if (res.success && res.data) {
          const post = res.data.post;
          setCanonicalPostId(post.post_id);
          setTitle(post.post_title);
          setIsPublished(post.post_is_published);
          setInitialPublished(post.post_is_published);
          const markdownFromMetadata = markdownContentFromMetadata(
            post.post_metadata,
          );
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
          setError(t("blog.post.failed_load"));
        }
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : t("blog.post.failed_load"));
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
    const request = {
      post_title: title(),
      post_content: body(),
      post_tags: postTags,
      post_is_published: isPublished(),
    } satisfies UpdatePostRequest;

    try {
      const res = await blogApi.updatePost(
        request,
        canonicalPostId() ?? params.post_id!,
      );
      if (res.success) {
        navigate(
          `/blog/${encodeURIComponent(res.data.post_slug || res.data.post_id)}`,
          {
            replace: true,
          },
        );
      } else {
        setError(t("blog.post.failed_update"));
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : t("blog.post.failed_update"));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <main class={pageStyles.page}>
      <div class={`${pageStyles.pageInner} flex flex-row gap-8`}>
        <div class="flex-1">
          <h2 class={`${pageStyles.titleSm} mb-4`}>
            {t("page.blog.edit_title")}
          </h2>

          <Show when={isLoading()}>
            <div class={pageStyles.muted}>{t("blog.loading_posts")}</div>
          </Show>

          <Show when={!isLoading()}>
            <form onSubmit={handleSubmit} class="flex flex-col gap-4">
              <input
                type="text"
                placeholder={t("blog.post.title_placeholder")}
                value={title()}
                onInput={(e) => setTitle(e.currentTarget.value)}
                required
                class={pageStyles.input}
              />
              <input
                type="text"
                placeholder={t("blog.post.tags_placeholder")}
                value={tags()}
                onInput={(e) => setTags(e.currentTarget.value)}
                class={pageStyles.input}
              />
              <label class="flex items-center gap-2 text-sm text-ink-muted">
                <input
                  type="checkbox"
                  checked={isPublished()}
                  onChange={(e) => setIsPublished(e.currentTarget.checked)}
                  class="h-4 w-4 rounded-sm border-line text-ink"
                />
                {t("blog.post.published")}
              </label>
              {!isPublished() && (
                <div class={pageStyles.muted}>
                  {t("blog.post.draft_visibility_edit")}
                </div>
              )}
              <div class="blog-editor w-full min-w-0 mb-8 relative z-0">
                <label class="font-medium text-ink mb-2 block">
                  {t("blog.post.content_markdown")}
                </label>
                <MarkdownEditor
                  value={body()}
                  onChange={setBody}
                  options={{ height: "28rem" }}
                />
              </div>
              {error() && <div class={pageStyles.alertError}>{error()}</div>}
              <div class="blog-form-actions flex gap-4 relative z-10">
                <button
                  type="submit"
                  disabled={isSubmitting()}
                  class={pageStyles.buttonPrimary}
                >
                  {isSubmitting()
                    ? isPublished()
                      ? t("blog.post.updating")
                      : t("common.saving")
                    : isPublished() && initialPublished() === false
                      ? t("blog.post.publish")
                      : isPublished()
                        ? t("blog.post.update")
                        : t("blog.post.save_draft")}
                </button>
                <button
                  type="button"
                  onClick={() => navigate(`/blog/${params.post_id}`)}
                  class={pageStyles.buttonSecondary}
                >
                  {t("common.cancel")}
                </button>
              </div>
            </form>
          </Show>
        </div>
      </div>
    </main>
  );
}
