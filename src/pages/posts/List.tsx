import { A, useNavigate, useSearchParams } from "@solidjs/router";
import {
  createResource,
  Show,
  For,
  createSignal,
  createEffect,
  onCleanup,
} from "solid-js";
import { blogApi } from "../../services/all_api";
import type { PostInfo } from "../../dtos/responses/blog";
import { isSuperuser, user } from "../../state/auth";
import { pageStyles } from "../../styles/pageStyles";
import { UserBadge } from "../../components/UserBadge";

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
  const [searchQuery, setSearchQuery] = createSignal(
    getParamString(searchParams.q),
  );
  const [searchType, setSearchType] = createSignal<"title" | "tag">(
    (getParamString(searchParams.type) as "title" | "tag") || "title",
  );
  const [debouncedQuery, setDebouncedQuery] = createSignal("");
  const [tagInput, setTagInput] = createSignal("");
  const [selectedTags, setSelectedTags] = createSignal<string[]>(
    parseTagsParam(searchParams.tags),
  );
  const initialPage = Math.max(
    1,
    Number.parseInt(getParamString(searchParams.page) || "1", 10) || 1,
  );
  const [page, setPage] = createSignal(initialPage);
  const [availablePages, setAvailablePages] = createSignal(1);

  // Debounce search input
  let debounceTimer: ReturnType<typeof setTimeout>;
  onCleanup(() => clearTimeout(debounceTimer));
  createEffect(() => {
    const query = searchQuery();
    const type = searchType();
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      setDebouncedQuery(query);
      if (query) {
        setSearchParams({ q: query, type });
      } else {
        setSearchParams({ q: undefined, type: undefined });
      }
    }, 300);
  });

  createEffect(() => {
    const tags = selectedTags();
    if (tags.length > 0) {
      setSearchParams({ tags: tags.join(",") });
    } else {
      setSearchParams({ tags: undefined });
    }
  });

  createEffect(() => {
    const currentPage = page();
    if (currentPage > 1) {
      setSearchParams({ page: String(currentPage) });
    } else {
      setSearchParams({ page: undefined });
    }
  });

  let initializedFilters = false;
  createEffect(() => {
    debouncedQuery();
    searchType();
    selectedTags();
    if (initializedFilters) {
      setPage(1);
      setSearchParams({ page: undefined });
    } else {
      initializedFilters = true;
    }
  });

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

  // Fetch posts or search results based on query
  const [posts, { refetch }] = createResource(
    () => ({
      query: debouncedQuery(),
      type: searchType(),
      tags: selectedTags(),
      page: page(),
    }),
    async ({ query, type, tags, page }) => {
      const trimmedQuery = query.trim();
      const activeTags = tags.map(normalizeTag).filter(Boolean);

      if (trimmedQuery || activeTags.length > 0) {
        return blogApi.searchPosts(
          trimmedQuery,
          type,
          page,
          PAGE_SIZE,
          activeTags,
        );
      }

      return blogApi.getPosts({ page, posts_per_page: PAGE_SIZE });
    },
  );

  const navigate = useNavigate();
  const [displayPosts, setDisplayPosts] = createSignal<PostInfo[]>([]);
  createEffect(() => {
    const data = posts()?.data;
    if (data?.posts !== undefined) {
      setDisplayPosts(data.posts);
    }
    if (data && "available_pages" in data) {
      setAvailablePages(data.available_pages ?? 1);
    }
  });
  createEffect(() => {
    const totalPages = availablePages();
    if (totalPages > 0 && page() > totalPages) {
      setPage(totalPages);
    }
  });
  const postItems = () => displayPosts();

  // Search by tag when clicking a tag badge
  const searchByTag = (tag: string) => {
    addTag(tag);
  };

  const handleDeletePost = async (e: Event, postId: string) => {
    e.preventDefault();
    if (!confirm("Are you sure you want to delete this post?")) return;
    try {
      await blogApi.deletePost(postId);
      refetch();
    } catch (e) {
      alert("Failed to delete post: " + e);
    }
  };

  return (
    <main class={pageStyles.page}>
      <div class={pageStyles.pageInner}>
        {/* 1. LAYOUT FIX: Align title and button in a row */}
        <div class="flex flex-row items-center justify-between mb-4">
          <h1 class={pageStyles.titleSm}>Blog Posts</h1>
          <Show when={isSuperuser()}>
            <button
              class={pageStyles.buttonPrimary}
              onClick={() => navigate("/blog/new")}
            >
              + New Post
            </button>
          </Show>
        </div>

        <hr class={`${pageStyles.divider} mb-4`} />

        {/* Search UI */}
        <div class="flex flex-col sm:flex-row gap-2 mb-6">
          <div class="flex-1 relative">
            <input
              type="text"
              placeholder="Search posts..."
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.currentTarget.value)}
              class="w-full px-3 py-2 pr-10 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <Show when={searchQuery()}>
              <button
                onClick={() => {
                  setSearchQuery("");
                  setDebouncedQuery("");
                  setSearchParams({ q: undefined, type: undefined });
                }}
                class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
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
          <div class="flex rounded-lg border border-slate-300 dark:border-slate-600 overflow-hidden">
            <button
              onClick={() => setSearchType("title")}
              class={`px-4 py-2 text-sm font-medium transition-colors ${
                searchType() === "title"
                  ? "bg-blue-500 text-white"
                  : "bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
              }`}
            >
              Title
            </button>
            <button
              onClick={() => setSearchType("tag")}
              class={`px-4 py-2 text-sm font-medium transition-colors border-l border-slate-300 dark:border-slate-600 ${
                searchType() === "tag"
                  ? "bg-blue-500 text-white"
                  : "bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
              }`}
            >
              Tag
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
                  placeholder="Add a tag and press Enter..."
                  value={tagInput()}
                  onInput={(e) => setTagInput(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      addTag(tagInput());
                    }
                  }}
                  class="w-full px-3 py-2 pr-20 border border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
                <button
                  onClick={() => addTag(tagInput())}
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-xs px-2 py-1 rounded bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-slate-600"
                >
                  Add
                </button>
              </div>
              <Show when={selectedTags().length > 0}>
                <button
                  onClick={clearTags}
                  class="text-xs px-2 py-1 rounded border border-slate-300 dark:border-slate-600 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
                >
                  Clear tags
                </button>
              </Show>
            </div>
            <Show when={selectedTags().length > 0}>
              <div class="flex flex-wrap gap-1.5">
                <For each={selectedTags()}>
                  {(tag) => (
                    <button
                      onClick={() => removeTag(tag)}
                      class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300 hover:bg-blue-200 dark:hover:bg-blue-800/60 transition-colors cursor-pointer"
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
          <div class="mb-4 text-sm text-slate-500 dark:text-slate-400">
            Showing results for "{debouncedQuery()}" (
            {searchType() === "tag" ? "tag" : "title"} search)
          </div>
        </Show>

        <Show when={posts.loading && postItems().length === 0}>
          <div class={`${pageStyles.muted} p-4 text-center`}>
            Loading posts...
          </div>
        </Show>

        <Show when={posts.loading && postItems().length > 0}>
          <div class={`${pageStyles.muted} mb-2 text-xs`}>
            Updating results...
          </div>
        </Show>

        <Show when={posts.error}>
          <div class={pageStyles.alertError}>
            Error loading posts: {String(posts.error)}
          </div>
        </Show>

        <Show when={!posts.loading && !posts.error && postItems().length === 0}>
          <div class={`${pageStyles.cardPadded} text-center`}>
            <div class="text-base font-semibold text-slate-900 dark:text-slate-100">
              No posts yet
            </div>
            <p class={`${pageStyles.muted} mt-1`}>
              Check back soon, or create the first one.
            </p>
          </div>
        </Show>

        <Show when={postItems().length > 0}>
          <ul class="flex flex-col gap-4">
            <For each={postItems()}>
              {(post) => (
                <li
                  class={`${pageStyles.card} overflow-hidden transition hover:shadow-md`}
                >
                  <div class="flex">
                    <div class="flex flex-col items-center justify-center w-16 bg-slate-50 dark:bg-slate-800/60 border-r border-slate-200/80 dark:border-slate-800 rounded-l">
                      <span class="text-sm font-bold text-slate-700 dark:text-slate-200">
                        {(post.total_upvotes ?? 0) -
                          (post.total_downvotes ?? 0)}
                      </span>
                    </div>

                    {/* Content Column */}
                    <div class="flex-1 px-4 py-3">
                      <div class="text-xs text-slate-500 mb-1 flex items-center gap-1">
                        <UserBadge
                          userName={post.user_name ?? "Unknown"}
                          profilePictureUrl={post.user_profile_picture_url}
                          countryFlag={post.user_country_flag}
                          size="sm"
                        />
                        <span class="text-slate-400">•</span>
                        <span>
                          {new Date(post.post_created_at).toLocaleDateString()}
                        </span>
                        <span class="text-slate-400">•</span>
                        <span>{post.post_view_count ?? 0} views</span>
                        <span class="text-slate-400">•</span>
                        <span>{post.post_share_count ?? 0} shares</span>
                        <Show when={!post.post_is_published}>
                          <span class="ml-2 rounded-full bg-amber-100 px-2 py-0.5 text-[0.65rem] font-semibold uppercase tracking-wide text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">
                            Draft
                          </span>
                        </Show>
                        <Show
                          when={
                            user()?.user_info?.user_id &&
                            post.user_id === user()?.user_info?.user_id
                          }
                        >
                          <button
                            class={`${pageStyles.buttonGhost} ml-auto text-rose-600 dark:text-rose-400`}
                            onClick={(e) => handleDeletePost(e, post.post_id)}
                          >
                            Delete
                          </button>
                        </Show>
                      </div>

                      <A
                        href={`/blog/${encodeURIComponent(post.post_slug || post.post_id)}`}
                        class="block text-lg font-semibold text-slate-900 dark:text-slate-100 hover:text-slate-700 dark:hover:text-slate-300 decoration-2 hover:underline underline-offset-2"
                      >
                        {post.post_title}
                      </A>

                      {/* Tag badges */}
                      <Show when={post.post_tags && post.post_tags.length > 0}>
                        <div class="flex flex-wrap gap-1.5 mt-2">
                          <For each={post.post_tags}>
                            {(tag) => (
                              <button
                                onClick={(e) => {
                                  e.preventDefault();
                                  searchByTag(tag);
                                }}
                                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300 hover:bg-blue-200 dark:hover:bg-blue-800/60 transition-colors cursor-pointer"
                              >
                                #{tag}
                              </button>
                            )}
                          </For>
                        </div>
                      </Show>
                    </div>
                  </div>
                </li>
              )}
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
              Prev
            </button>
            <div class="text-sm text-slate-600 dark:text-slate-300">
              Page {page()} of {availablePages()}
            </div>
            <button
              class={pageStyles.buttonSecondary}
              disabled={page() >= availablePages()}
              onClick={() => setPage((p) => Math.min(availablePages(), p + 1))}
            >
              Next
            </button>
          </div>
        </Show>
      </div>
    </main>
  );
}
