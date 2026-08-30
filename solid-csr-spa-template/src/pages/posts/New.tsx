import { createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { blogApi } from "../../services/all_api";
import type { SubmitPostRequest } from "../../generated";
import MarkdownEditor from "../../components/MarkdownEditor";
import { pageStyles } from "../../styles/pageStyles";
import { t } from "../../state/i18n";
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
    const request = {
      post_title: title(),
      post_content: body(),
      post_tags: postTags,
      post_is_published: isPublished(),
    } satisfies SubmitPostRequest;

    try {
      const res = await blogApi.submitPost(request);
      if (res.success) {
        navigate(
          `/blog/${encodeURIComponent(res.data.post_slug || res.data.post_id)}`,
          {
            replace: true,
          },
        );
      } else {
        setError(t("blog.post.failed_publish"));
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : t("blog.post.failed_submit"));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <main class={pageStyles.page}>
      <div class={`${pageStyles.pageInner} flex flex-row gap-8`}>
        <div class="flex-1">
          <h2 class={`${pageStyles.titleSm} mb-4`}>
            {t("page.blog.new_title")}
          </h2>
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
              {t("blog.post.publish_immediately")}
            </label>
            {!isPublished() && (
              <div class={pageStyles.muted}>
                {t("blog.post.draft_visibility_new")}
              </div>
            )}
            <div class="w-full min-w-0 mb-8 relative z-0">
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
            <div class="flex gap-4 relative z-10">
              <button
                type="submit"
                disabled={isSubmitting()}
                class={pageStyles.buttonPrimary}
              >
                {isSubmitting()
                  ? isPublished()
                    ? t("blog.post.publishing")
                    : t("common.saving")
                  : isPublished()
                    ? t("blog.post.publish")
                    : t("blog.post.save_draft")}
              </button>
              <button
                type="button"
                onClick={() => navigate("/blog")}
                class={pageStyles.buttonSecondary}
              >
                {t("common.cancel")}
              </button>
            </div>
          </form>
        </div>
        {/* Optionally, place a sidebar here if you want Reddit-style right column */}
      </div>
    </main>
  );
}
