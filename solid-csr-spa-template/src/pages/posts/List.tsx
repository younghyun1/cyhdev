import { useNavigate, useSearchParams } from "@solidjs/router";
import {
  Show,
  For,
  createMemo,
  createSignal,
  createEffect,
  isPending,
  onCleanup,
  refresh,
  untrack,
} from "solid-js";
import { blogApi } from "../../services/all_api";
import type { PostInfoWithVote } from "../../generated";
import { isSuperuser, user } from "../../state/auth";
import { pageStyles } from "../../styles/pageStyles";
import { UserBadge } from "../../components/UserBadge";
import { t, tx } from "../../state/i18n";

// Helper to normalize search param (can be string | string[] | undefined)
const getParamString = (param: string | string[] | undefined): string => {
  if (Array.isArray(param)) return param[0] ?? "";
  return param ?? "";
};

const parseTagsParam = (param: string | string[] | undefined): string[] => {
  const raw = getParamString(param);
  return raw
    .split(",")
    .map((tag) => tag.trim())
    .filter(Boolean);
};

const normalizeTag = (tag: string): string => tag.trim().toLowerCase();

const PAGE_SIZE = 20;

export default function PostsList() {
  const [searchParams, setSearchParams] = useSearchParams();
  const initialFilters = untrack(() => ({
    query: getParamString(searchParams.q),
    type:
      (getParamString(searchParams.type) as "title" | "tag") || "title",
    tags: parseTagsParam(searchParams.tags),
    page: Math.max(
      1,
      Number.parseInt(getParamString(searchParams.page) || "1", 10) || 1,
    ),
  }));
  const [searchQuery, setSearchQuery] = createSignal(initialFilters.query);
  const [searchType, setSearchType] = createSignal<"title" | "tag">(
    initialFilters.type,
  );
  const [debouncedQuery, setDebouncedQuery] = createSignal("");
  const [tagInput, setTagInput] = createSignal("");
  const [selectedTags, setSelectedTags] = createSignal<string[]>(
    initialFilters.tags,
  );
  const [page, setPage] = createSignal(initialFilters.page);
  const [availablePages, setAvailablePages] = createSignal(1);

  // Debounce search input
  let debounceTimer: ReturnType<typeof setTimeout>;
  onCleanup(() => clearTimeout(debounceTimer));
  createEffect(
    () => ({ query: searchQuery(), type: searchType() }),
    ({ query, type }) => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        setDebouncedQuery(query);
        if (query) {
          setSearchParams({ q: query, type });
        } else {
          setSearchParams({ q: undefined, type: undefined });
        }
      }, 300);
    },
  );

  createEffect(
    () => selectedTags(),
    (tags) => {
      if (tags.length > 0) {
        setSearchParams({ tags: tags.join(",") });
      } else {
        setSearchParams({ tags: undefined });
      }
    },
  );

  createEffect(
    () => page(),
    (currentPage) => {
      if (currentPage > 1) {
        setSearchParams({ page: String(currentPage) });
      } else {
        setSearchParams({ page: undefined });
      }
    },
  );

  let initializedFilters = false;
  createEffect(
    () => [debouncedQuery(), searchType(), selectedTags()] as const,
    () => {
      if (initializedFilters) {
        setPage(1);
        setSearchParams({ page: undefined });
      } else {
        initializedFilters = true;
      }
    },
  );

  const addTag = (tag: string) => {
    const normalized = normalizeTag(tag);
    if (!normalized) return;
    setSelectedTags((prev) =>
      prev.includes(normalized) ? prev : [...prev, normalized],
    );
    setTagInput("");
  };

  const removeTag = (tag: string) => {
    setSelectedTags((prev) => prev.filter((t) => t !== tag));
  };

  const clearTags = () => setSelectedTags([]);

  // Fetch posts or search results based on query. Errors resolve to a marker
  // object (rather than an Errored boundary) so the list UI can keep showing
  // the previous results alongside an inline error message.
  const posts = createMemo(async () => {
    const query = debouncedQuery();
    const type = searchType();
    const tags = selectedTags();
    const currentPage = page();

    const trimmedQuery = query.trim();
    const activeTags = tags.map(normalizeTag).filter(Boolean);

    try {
      const res =
        trimmedQuery || activeTags.length > 0
          ? await blogApi.searchPosts(
              trimmedQuery,
              type,
              currentPage,
              PAGE_SIZE,
              activeTags,
            )
          : await blogApi.getPosts({
              page: currentPage,
              posts_per_page: PAGE_SIZE,
            });
      return { ok: true as const, res };
    } catch (err) {
      return { ok: false as const, error: String(err) };
    }
  });
  const postsPending = () => isPending(() => posts());

  const navigate = useNavigate();
  const [displayPosts, setDisplayPosts] = createSignal<
    ReadonlyArray<PostInfoWithVote>
  >([]);
  const [loadError, setLoadError] = createSignal<string | null>(null);
  createEffect(
    () => posts(),
    (result) => {
      if (!result.ok) {
        setLoadError(result.error);
        return;
      }
      setLoadError(null);
      const data = result.res.data;
      if (data?.posts !== undefined) {
        setDisplayPosts(data.posts);
      }
      if (data && "available_pages" in data) {
        setAvailablePages(data.available_pages ?? 1);
      }
    },
  );
  createEffect(
    () => ({ totalPages: availablePages(), current: page() }),
    ({ totalPages, current }) => {
      if (totalPages > 0 && current > totalPages) {
        setPage(totalPages);
      }
    },
  );
  const postItems = () => displayPosts();

  // Search by tag when clicking a tag badge
  const searchByTag = (tag: string) => {
    addTag(tag);
  };

  const handleDeletePost = async (e: Event, postId: string) => {
    e.preventDefault();
    if (!confirm(t("blog.delete_post_confirm"))) return;
    try {
      await blogApi.deletePost(postId);
      refresh(posts);
    } catch (e) {
      alert(tx("blog.delete_post_failed", { error: String(e) }));
    }
  };

  return (
    <main class={pageStyles.page}>
      <div class={pageStyles.pageInner}>
        {/* 1. LAYOUT FIX: Align title and button in a row */}
        <div class="flex flex-row items-center justify-between mb-4">
          <h1 class={pageStyles.titleSm}>{t("page.blog.list_title")}</h1>
          <Show when={isSuperuser()}>
            <button
              class={pageStyles.buttonPrimary}
              onClick={() => navigate("/blog/new")}
            >
              {t("blog.new_post")}
            </button>
          </Show>
        </div>

        <hr class={`${pageStyles.divider} mb-4`} />

        {/* Search UI */}
        <div class="flex flex-col sm:flex-row gap-2 mb-6">
          <div class="flex-1 relative">
            <input
              type="text"
              placeholder={t("blog.search_placeholder")}
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.currentTarget.value)}
              class={`${pageStyles.input} pr-10`}
            />
            <Show when={searchQuery()}>
              <button
                onClick={() => {
                  setSearchQuery("");
                  setDebouncedQuery("");
                  setSearchParams({ q: undefined, type: undefined });
                }}
                class="absolute right-2 top-1/2 -translate-y-1/2 text-ink-faint hover:text-ink-muted"
              >
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </Show>
          </div>
          <div class="flex rounded-sm border border-line overflow-hidden">
            <button
              onClick={() => setSearchType("title")}
              class={[
                "px-4 py-2 text-sm font-medium transition-colors",
                searchType() === "title"
                  ? "bg-accent text-paper"
                  : "bg-surface text-ink-muted hover:bg-surface-2",
              ]}
            >
              {t("blog.search_title")}
            </button>
            <button
              onClick={() => setSearchType("tag")}
              class={[
                "px-4 py-2 text-sm font-medium transition-colors border-l border-line",
                searchType() === "tag"
                  ? "bg-accent text-paper"
                  : "bg-surface text-ink-muted hover:bg-surface-2",
              ]}
            >
              {t("blog.search_tag")}
            </button>
          </div>
        </div>

        {/* Tag filters */}
        <div class="mb-6">
          <div class="flex flex-col gap-2">
            <div class="flex flex-wrap gap-2 items-center">
              <div class="flex-1 relative">
                <input
                  type="text"
                  placeholder={t("blog.tag_placeholder")}
                  value={tagInput()}
                  onInput={(e) => setTagInput(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      addTag(tagInput());
                    }
                  }}
                  class={`${pageStyles.input} pr-20`}
                />
                <button
                  onClick={() => addTag(tagInput())}
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-xs px-2 py-1 rounded-sm bg-surface-2 text-ink-muted hover:text-ink"
                >
                  {t("blog.add_tag")}
                </button>
              </div>
              <Show when={selectedTags().length > 0}>
                <button
                  onClick={clearTags}
                  class="text-xs px-2 py-1 rounded-sm border border-line text-ink-muted hover:bg-surface-2"
                >
                  {t("blog.clear_tags")}
                </button>
              </Show>
            </div>
            <Show when={selectedTags().length > 0}>
              <div class="flex flex-wrap gap-1.5">
                <For each={selectedTags()}>
                  {(tag) => (
                    <button
                      onClick={() => removeTag(tag)}
                      class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full font-mono text-xs font-medium bg-accent-soft text-accent hover:opacity-80 transition-opacity cursor-pointer"
                    >
                      #{tag}
                      <span class="text-[0.6rem]">×</span>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>

        <Show when={debouncedQuery()}>
          <div class="mb-4 text-sm text-ink-muted">
            {tx("blog.showing_results", {
              query: debouncedQuery(),
              type:
                searchType() === "tag"
                  ? t("blog.search_tag").toLowerCase()
                  : t("blog.search_title").toLowerCase(),
            })}
          </div>
        </Show>

        <Show when={postsPending() && postItems().length === 0}>
          <div class={`${pageStyles.muted} p-4 text-center`}>
            {t("blog.loading_posts")}
          </div>
        </Show>

        <Show when={postsPending() && postItems().length > 0}>
          <div class={`${pageStyles.muted} mb-2 text-xs`}>
            {t("blog.updating_results")}
          </div>
        </Show>

        <Show when={loadError()}>
          <div class={pageStyles.alertError}>
            {tx("blog.error_loading_posts", { error: loadError() ?? "" })}
          </div>
        </Show>

        <Show when={!postsPending() && !loadError() && postItems().length === 0}>
          <div class={`${pageStyles.cardPadded} text-center`}>
            <div class="text-base font-semibold text-ink">
              {t("blog.no_posts_title")}
            </div>
            <p class={`${pageStyles.muted} mt-1`}>
              {t("blog.no_posts_subtitle")}
            </p>
          </div>
        </Show>

        <Show when={postItems().length > 0}>
          <ul class="flex flex-col">
            <For each={postItems()}>
              {(post) => {
                const score = () =>
                  (post.total_upvotes ?? 0) - (post.total_downvotes ?? 0);
                return (
                  <li class="group border-b border-line py-4 first:border-t">
                    <div class="flex flex-wrap items-center gap-x-2 gap-y-1 font-mono tabular-nums text-xs text-ink-muted">
                      <UserBadge
                        userName={post.user_name ?? t("common.unknown")}
                        profilePictureUrl={post.user_profile_picture_url}
                        countryFlag={post.user_country_flag}
                        size="sm"
                      />
                      <span class="text-ink-faint">·</span>
                      <span>
                        {new Date(post.post_created_at).toLocaleDateString()}
                      </span>
                      <span class="text-ink-faint">·</span>
                      <span>
                        {score() > 0 ? `+${score()}` : score()}
                      </span>
                      <span class="text-ink-faint">·</span>
                      <span>
                        {post.post_view_count ?? 0} {t("common.views")}
                      </span>
                      <span class="text-ink-faint">·</span>
                      <span>
                        {post.post_share_count ?? 0} {t("common.shares")}
                      </span>
                      <Show when={!post.post_is_published}>
                        <span class="rounded-full bg-accent-soft px-2 py-0.5 text-[0.65rem] font-semibold uppercase tracking-wide text-accent">
                          {t("common.draft")}
                        </span>
                      </Show>
                      <Show
                        when={
                          user()?.user_info?.user_id &&
                          post.user_id === user()?.user_info?.user_id
                        }
                      >
                        <button
                          class={`${pageStyles.buttonGhost} ml-auto py-0.5 text-danger`}
                          onClick={(e) => handleDeletePost(e, post.post_id)}
                        >
                          {t("common.delete")}
                        </button>
                      </Show>
                    </div>

                    <a
                      href={`/blog/${encodeURIComponent(post.post_slug || post.post_id)}`}
                      class="mt-1 block text-lg font-semibold text-ink group-hover:text-accent decoration-accent/40 underline-offset-4 hover:underline transition-colors"
                    >
                      {post.post_title}
                    </a>

                    {/* Tags: inline mono links, no pills */}
                    <Show when={post.post_tags && post.post_tags.length > 0}>
                      <div class="flex flex-wrap gap-x-3 gap-y-1 mt-1.5 font-mono text-xs">
                        <For each={post.post_tags}>
                          {(tag) => (
                            <button
                              onClick={(e) => {
                                e.preventDefault();
                                searchByTag(tag);
                              }}
                              class="text-accent/80 hover:text-accent hover:underline underline-offset-4 transition-colors cursor-pointer"
                            >
                              #{tag}
                            </button>
                          )}
                        </For>
                      </div>
                    </Show>
                  </li>
                );
              }}
            </For>
          </ul>
        </Show>

        <Show when={availablePages() > 1}>
          <div class="mt-6 flex items-center justify-between">
            <button
              class={pageStyles.buttonSecondary}
              disabled={page() <= 1}
              onClick={() => setPage((p) => Math.max(1, p - 1))}
            >
              {t("blog.prev")}
            </button>
            <div class="font-mono tabular-nums text-sm text-ink-muted">
              {tx("blog.page_of", { page: page(), pages: availablePages() })}
            </div>
            <button
              class={pageStyles.buttonSecondary}
              disabled={page() >= availablePages()}
              onClick={() => setPage((p) => Math.min(availablePages(), p + 1))}
            >
              {t("common.next")}
            </button>
          </div>
        </Show>
      </div>
    </main>
  );
}
