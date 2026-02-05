import { A, useNavigate } from "@solidjs/router";
import { createResource, Show, For } from "solid-js"; // Import For
import { blogApi } from "../../services/all_api";
import { isSuperuser, user } from "../../state/auth";
import { pageStyles } from "../../styles/pageStyles";

export default function PostsList() {
  const [posts, { refetch }] = createResource(() => blogApi.getPosts());
  const navigate = useNavigate();
  const postItems = () => posts()?.data?.posts ?? [];

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

        <hr class={`${pageStyles.divider} mb-6`} />

        <Show when={posts.loading}>
          <div class={`${pageStyles.muted} p-4 text-center`}>
            Loading posts...
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
                        {((post as any)?.total_upvotes ?? 0) -
                          ((post as any)?.total_downvotes ?? 0)}
                      </span>
                    </div>

                    {/* Content Column */}
                    <div class="flex-1 px-4 py-3">
                      <div class="text-xs text-slate-500 mb-1 flex items-center gap-1">
                        <Show when={(post as any).user_profile_picture_url}>
                          <img
                            src={(post as any).user_profile_picture_url}
                            alt={(post as any).user_name}
                            class="w-5 h-5 rounded-full object-cover border border-slate-200 dark:border-slate-700"
                          />
                        </Show>
                        <span class="font-medium text-slate-900 dark:text-slate-300">
                          {(post as any).user_name ?? "Unknown"}
                        </span>
                        <span class="text-slate-400">•</span>
                        <span>
                          {new Date(post.post_created_at).toLocaleDateString()}
                        </span>
                        <span class="text-slate-400">•</span>
                        <span>{(post as any).post_view_count ?? 0} views</span>
                        <span class="text-slate-400">•</span>
                        <span>
                          {(post as any).post_share_count ?? 0} shares
                        </span>
                        <Show when={!post.post_is_published}>
                          <span class="ml-2 rounded-full bg-amber-100 px-2 py-0.5 text-[0.65rem] font-semibold uppercase tracking-wide text-amber-800 dark:bg-amber-900/40 dark:text-amber-200">
                            Draft
                          </span>
                        </Show>
                        <Show
                          when={
                            user()?.user_info?.user_id &&
                            (post as any).user_id === user()?.user_info?.user_id
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
                        href={`/blog/${post.post_id}`}
                        class="block text-lg font-semibold text-slate-900 dark:text-slate-100 hover:text-slate-700 dark:hover:text-slate-300 decoration-2 hover:underline underline-offset-2"
                      >
                        {post.post_title}
                      </A>
                    </div>
                  </div>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </div>
    </main>
  );
}
