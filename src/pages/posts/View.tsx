import {
  Show,
  createSignal,
  createResource,
  For,
  createMemo,
  createEffect,
} from "solid-js";
import { createStore } from "solid-js/store";
import { Key } from "@solid-primitives/keyed";
import { useParams, useNavigate, A } from "@solidjs/router";
import { blogApi } from "../../services/all_api";
import type { CommentResponse } from "../../dtos/responses/blog";
import { isAuthenticated, user } from "../../state/auth";
import { pageStyles } from "../../styles/pageStyles";
import { UserBadge } from "../../components/UserBadge";
import { t, tx } from "../../state/i18n";
import DOMPurify from "dompurify";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import sql from "highlight.js/lib/languages/sql";
import typescript from "highlight.js/lib/languages/typescript";
import toml from "highlight.js/lib/languages/ini";
import html from "highlight.js/lib/languages/xml";
import css from "highlight.js/lib/languages/css";
import dockerfile from "highlight.js/lib/languages/dockerfile";
// Code-block colors come from the site theme (src/styles/code.css),
// imported globally via index.css.

hljs.registerLanguage("rust", rust);
hljs.registerLanguage("sql", sql);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("toml", toml);
hljs.registerLanguage("html", html);
hljs.registerLanguage("css", css);
hljs.registerLanguage("dockerfile", dockerfile);

// VoteState Mapping based on Rust Enum:
// 0: Upvoted
// 1: Downvoted
// 2: DidNotVote
type VoteState = 0 | 1 | 2;

export default function PostViewPage() {
  const params = useParams();
  const navigate = useNavigate();
  // Route param: either a slug or a UUID. Used only to fetch the post; the
  // read endpoint resolves both. Do not use this for mutations.
  const routeKey = () => params.post_id;
  // Canonical post UUID, resolved from the loaded post. All mutations (votes,
  // comments, delete) must use this: the comment/vote/delete endpoints parse
  // post_id strictly as a UUID and reject slugs. Falls back to the route key
  // before the post loads, but mutation UI is only reachable after load.
  const postId = () => postResource()?.post.post_id ?? params.post_id;
  const [postLoadError, setPostLoadError] = createSignal<string | null>(null);
  const [commentValue, setCommentValue] = createSignal("");
  const [commentLoading, setCommentLoading] = createSignal(false);
  const [commentError, setCommentError] = createSignal<string | null>(null);
  const [commentSort, setCommentSort] = createSignal<
    "best" | "top" | "new" | "old"
  >("best");

  const [postResource, { refetch }] = createResource(routeKey, async (pid) => {
    if (!pid) return null;
    setPostLoadError(null);

    try {
      const res = await blogApi.readPost(pid);
      return res?.data ?? null;
    } catch (err: unknown) {
      const status =
        typeof err === "object" &&
        err !== null &&
        "status" in err &&
        typeof (err as { status?: unknown }).status === "number"
          ? (err as { status: number }).status
          : undefined;
      const message =
        err instanceof Error ? err.message : t("blog.post.failed_load");

      if (status === 400 || status === 404) {
        navigate("/under-construction", { replace: true });
        return null;
      }

      setPostLoadError(message || t("blog.post.failed_load"));
      return null;
    }
  });

  // Store for optimistic vote states (Post + Comments)
  const [optimisticVotes, setOptimisticVotes] = createStore<{
    post?: {
      total_upvotes: number;
      total_downvotes: number;
      vote_state: VoteState;
    };
    comments: Record<
      string,
      {
        total_upvotes: number;
        total_downvotes: number;
        vote_state: VoteState;
      }
    >;
  }>({ comments: {} });

  // Store for locally added comments (optimistic replies)
  const [localComments, setLocalComments] = createStore<
    Record<string, CommentResponse>
  >({});

  // Per-comment reply state
  const [replyOpen, setReplyOpen] = createStore<Record<string, boolean>>({});
  const [replyText, setReplyText] = createStore<Record<string, string>>({});
  const [replyLoading, setReplyLoading] = createStore<Record<string, boolean>>(
    {},
  );
  const [replyError, setReplyError] = createStore<
    Record<string, string | null>
  >({});

  const [editOpen, setEditOpen] = createStore<Record<string, boolean>>({});
  const [editText, setEditText] = createStore<Record<string, string>>({});
  const [editLoading, setEditLoading] = createStore<Record<string, boolean>>(
    {},
  );
  const [editError, setEditError] = createStore<Record<string, string | null>>(
    {},
  );

  const handleDeletePost = async () => {
    if (!confirm(t("blog.delete_post_confirm"))) return;
    try {
      await blogApi.deletePost(postId()!);
      navigate("/blog");
    } catch (e) {
      alert(tx("blog.delete_post_failed", { error: String(e) }));
    }
  };

  const handleDeleteComment = async (commentId: string) => {
    if (!confirm(t("blog.comments.delete_confirm"))) return;
    try {
      await blogApi.deleteComment(postId()!, commentId);
      refetch();
    } catch (e) {
      alert(tx("blog.comments.delete_failed", { error: String(e) }));
    }
  };

  const handleVote = async (
    type: "post" | "comment",
    isUpvote: boolean,
    ids: { postId: string | undefined; commentId?: string },
  ) => {
    if (!ids.postId) return;
    const originalPostState = postResource()?.post;
    const originalCommentState =
      type === "comment" && ids.commentId
        ? (postResource()?.comments.find(
            (c) => c.comment_id === ids.commentId,
          ) ?? localComments[ids.commentId])
        : undefined;

    if (type === "post" && !originalPostState) return;
    if (type === "comment" && !originalCommentState) return;

    // Determine current state from optimistic store or initial data
    const currentState =
      type === "post"
        ? (optimisticVotes.post?.vote_state ?? postResource()?.vote_state)
        : (optimisticVotes.comments[ids.commentId!]?.vote_state ??
          originalCommentState?.vote_state);

    const currentUpvotes =
      (type === "post"
        ? (optimisticVotes.post?.total_upvotes ??
          originalPostState?.total_upvotes)
        : (optimisticVotes.comments[ids.commentId!]?.total_upvotes ??
          originalCommentState?.total_upvotes)) || 0;
    const currentDownvotes =
      (type === "post"
        ? (optimisticVotes.post?.total_downvotes ??
          originalPostState?.total_downvotes)
        : (optimisticVotes.comments[ids.commentId!]?.total_downvotes ??
          originalCommentState?.total_downvotes)) || 0;

    const isRescinding =
      (isUpvote && currentState === 0) || (!isUpvote && currentState === 1);

    // --- Optimistic Update ---
    let newVoteState: VoteState = isUpvote ? 0 : 1;
    let newUpvotes = currentUpvotes;
    let newDownvotes = currentDownvotes;

    if (isRescinding) {
      newVoteState = 2; // DidNotVote
      if (isUpvote) newUpvotes--;
      else newDownvotes--;
    } else {
      if (currentState === 0) newUpvotes--; // changing from upvote
      if (currentState === 1) newDownvotes--; // changing from downvote
      if (isUpvote) newUpvotes++;
      else newDownvotes++;
    }

    const optimisticUpdate = {
      total_upvotes: newUpvotes,
      total_downvotes: newDownvotes,
      vote_state: newVoteState,
    };
    if (type === "post") {
      setOptimisticVotes("post", optimisticUpdate);
    } else {
      setOptimisticVotes("comments", ids.commentId!, optimisticUpdate);
    }

    // --- API Call ---
    try {
      if (isRescinding) {
        if (type === "post") await blogApi.rescindPostVote(ids.postId);
        else await blogApi.rescindCommentVote(ids.postId, ids.commentId!);
      } else {
        if (type === "post")
          await blogApi.votePost({ is_upvote: isUpvote }, ids.postId);
        else
          await blogApi.voteComment(
            { is_upvote: isUpvote },
            ids.postId,
            ids.commentId!,
          );
      }
      // Success! The optimistic update is already showing.
      // refetch(); // <-- REMOVED for smoother UX
    } catch (error) {
      console.error("Vote failed:", error);
      // Roll back optimistic update instead of refetching entire post
      const rollback = {
        total_upvotes: currentUpvotes,
        total_downvotes: currentDownvotes,
        vote_state: currentState as VoteState,
      };
      if (type === "post") {
        setOptimisticVotes("post", rollback);
      } else if (ids.commentId) {
        setOptimisticVotes("comments", ids.commentId, rollback);
      }
    }
  };

  const handleSubmitComment = async (e: Event) => {
    e.preventDefault();
    setCommentLoading(true);
    setCommentError(null);
    try {
      await blogApi.submitComment(
        {
          is_guest: !isAuthenticated(),
          guest_id: null,
          guest_password: null,
          parent_comment_id: null,
          comment_content: commentValue(),
        },
        postId()!,
      );
      setCommentValue("");
      refetch();
    } catch (err: unknown) {
      setCommentError(
        err instanceof Error ? err.message : t("blog.comments.failed_submit"),
      );
    } finally {
      setCommentLoading(false);
    }
  };

  // Reply handlers
  const toggleReply = (commentId: string) => {
    const current = !!replyOpen[commentId];
    setReplyOpen(commentId, !current);
    if (!current) {
      setReplyText(commentId, "");
      setReplyError(commentId, null);
    }
  };
  const handleSubmitReply = async (parentCommentId: string) => {
    const content = (replyText[parentCommentId] ?? "").trim();
    if (!content) return;
    setReplyLoading(parentCommentId, true);
    setReplyError(parentCommentId, null);
    try {
      const res = await blogApi.submitComment(
        {
          is_guest: !isAuthenticated(),
          guest_id: null,
          guest_password: null,
          parent_comment_id: parentCommentId,
          comment_content: content,
        },
        postId()!,
      );
      // Optimistically add the new reply locally so it appears immediately
      if (res?.data) {
        const newComment = res.data;
        setLocalComments(newComment.comment_id, newComment);
      }
      setReplyText(parentCommentId, "");
      setReplyOpen(parentCommentId, false);
      // No immediate refetch; the local comment will reconcile on future refetch
    } catch (err: unknown) {
      setReplyError(
        parentCommentId,
        err instanceof Error ? err.message : t("blog.comments.failed_reply"),
      );
    } finally {
      setReplyLoading(parentCommentId, false);
    }
  };

  const toggleEdit = (comment: {
    comment_id: string;
    comment_content: string;
  }) => {
    const current = !!editOpen[comment.comment_id];
    if (current) {
      setEditOpen(comment.comment_id, false);
      setEditText(comment.comment_id, "");
      setEditError(comment.comment_id, null);
    } else {
      setEditOpen(comment.comment_id, true);
      setEditText(comment.comment_id, comment.comment_content);
      setReplyOpen(comment.comment_id, false);
    }
  };

  const handleUpdateComment = async (commentId: string) => {
    const content = (editText[commentId] ?? "").trim();
    if (!content) return;
    setEditLoading(commentId, true);
    setEditError(commentId, null);
    try {
      await blogApi.updateComment(
        { comment_content: content },
        postId()!,
        commentId,
      );
      refetch();
      setEditOpen(commentId, false);
    } catch (err: unknown) {
      setEditError(
        commentId,
        err instanceof Error ? err.message : t("blog.comments.failed_update"),
      );
    } finally {
      setEditLoading(commentId, false);
    }
  };

  type CommentTreeNode = CommentResponse & { children: CommentTreeNode[] };

  function buildCommentTree(
    flatComments: CommentResponse[],
  ): CommentTreeNode[] {
    const commentsById: Record<string, CommentTreeNode> = {};
    const roots: CommentTreeNode[] = [];

    // Merge locally added comments (optimistic replies) without changing existing order
    const flat = [...flatComments];
    for (const id in localComments) {
      const local = localComments[id];
      if (local && !flat.find((c) => c.comment_id === id)) {
        flat.push(local);
      }
    }

    for (const c of flat) {
      commentsById[c.comment_id] = { ...c, children: [] };
    }
    for (const c of flat) {
      const parent = c.parent_comment_id
        ? commentsById[c.parent_comment_id]
        : undefined;
      const self = commentsById[c.comment_id]!;
      if (parent) {
        parent.children.push(self);
      } else {
        roots.push(self);
      }
    }
    return roots;
  }

  function getBaseCommentState(c: CommentTreeNode) {
    return {
      up: c.total_upvotes,
      down: c.total_downvotes,
      createdAt: new Date(c.comment_created_at).getTime(),
    };
  }
  function compareComments(a: CommentTreeNode, b: CommentTreeNode) {
    const sort = commentSort();
    const A = getBaseCommentState(a);
    const B = getBaseCommentState(b);
    switch (sort) {
      case "best": {
        const sa = A.up - A.down;
        const sb = B.up - B.down;
        if (sb !== sa) return sb - sa;
        return B.createdAt - A.createdAt;
      }
      case "top": {
        if (B.up !== A.up) return B.up - A.up;
        return B.createdAt - A.createdAt;
      }
      case "new":
        return B.createdAt - A.createdAt;
      case "old":
        return A.createdAt - B.createdAt;
      default:
        return 0;
    }
  }
  function sortCommentsTree(nodes: CommentTreeNode[]): CommentTreeNode[] {
    const copy = nodes.map((n) => ({
      ...n,
      children:
        n.children && n.children.length > 0 ? sortCommentsTree(n.children) : [],
    }));
    copy.sort(compareComments);
    return copy;
  }
  function renderComments(comments: CommentTreeNode[], depth = 0) {
    // Keyed by comment_id so a sort change reorders existing DOM nodes instead of
    // tearing the whole tree down and rebuilding it.
    return (
      <Key each={comments} by={(c) => c.comment_id}>
        {(comment) => {
          const voteState = () =>
            optimisticVotes.comments[comment().comment_id]?.vote_state ??
            comment().vote_state;
          const upvotes = () =>
            optimisticVotes.comments[comment().comment_id]?.total_upvotes ??
            comment().total_upvotes;
          const downvotes = () =>
            optimisticVotes.comments[comment().comment_id]?.total_downvotes ??
            comment().total_downvotes;

          return (
            <div
              class={`mt-2 pl-3 md:pl-4 border-l border-line`}
              style={{ "margin-left": `${depth * 16}px` }}
            >
              <div class="mb-1.5 flex flex-wrap items-center gap-2 text-xs text-ink-muted">
                <UserBadge
                  userName={comment().user_name ?? t("common.unknown")}
                  profilePictureUrl={comment().user_profile_picture_url}
                  countryFlag={comment().user_country_flag}
                  size="sm"
                />
                <span class="ml-3 text-xs">
                  {new Date(comment().comment_created_at).toLocaleString()}
                </span>
              </div>
              <Show
                when={editOpen[comment().comment_id]}
                fallback={
                  <div class="text-ink whitespace-pre-wrap">
                    {comment().comment_content}
                  </div>
                }
              >
                <div class="mt-2">
                  <textarea
                    class={`${pageStyles.textarea} min-h-20`}
                    value={editText[comment().comment_id] ?? ""}
                    onInput={(e) =>
                      setEditText(comment().comment_id, e.currentTarget.value)
                    }
                  />
                  <Show when={editError[comment().comment_id]}>
                    <div class="text-sm text-danger">
                      {editError[comment().comment_id]}
                    </div>
                  </Show>
                  <div class="mt-2 flex items-center gap-2">
                    <button
                      class={`${pageStyles.buttonPrimary} px-3 py-1 text-sm`}
                      disabled={
                        editLoading[comment().comment_id] ||
                        !(editText[comment().comment_id] ?? "").trim()
                      }
                      onClick={() => handleUpdateComment(comment().comment_id)}
                    >
                      {editLoading[comment().comment_id]
                        ? t("common.saving")
                        : t("common.save")}
                    </button>
                    <button
                      type="button"
                      class={`${pageStyles.buttonSecondary} px-3 py-1 text-sm`}
                      onClick={() => toggleEdit(comment())}
                    >
                      {t("common.cancel")}
                    </button>
                  </div>
                </div>
              </Show>
              <div class="flex items-center gap-2 mt-2 mb-1">
                <button
                  class={`text-lg px-1 ${voteState() === 0 ? "text-ok font-bold" : "text-ink-muted hover:text-ok"}`}
                  onClick={() =>
                    handleVote("comment", true, {
                      postId: postId(),
                      commentId: comment().comment_id,
                    })
                  }
                  title={t("blog.vote.upvote")}
                >
                  ▲
                </button>

                <span class="text-xs font-semibold tabular-nums text-ink">
                  {upvotes() - downvotes()}
                </span>

                <button
                  class={`text-lg px-1 ${voteState() === 1 ? "text-danger font-bold" : "text-ink-muted hover:text-danger"}`}
                  onClick={() =>
                    handleVote("comment", false, {
                      postId: postId(),
                      commentId: comment().comment_id,
                    })
                  }
                  title={t("blog.vote.downvote")}
                >
                  ▼
                </button>
              </div>
              <div class="mt-1 flex gap-3">
                <button
                  class={`${pageStyles.link} text-xs`}
                  onClick={() => toggleReply(comment().comment_id)}
                >
                  {t("blog.comments.reply")}
                </button>
                <Show
                  when={
                    user()?.user_info?.user_id &&
                    (comment().user_id === user()?.user_info?.user_id ||
                      postResource()?.post?.user_id ===
                        user()?.user_info?.user_id)
                  }
                >
                  <button
                    class={`${pageStyles.link} text-xs`}
                    onClick={() => toggleEdit(comment())}
                  >
                    {t("common.edit")}
                  </button>
                  <button
                    class="text-xs text-danger hover:underline"
                    onClick={() => handleDeleteComment(comment().comment_id)}
                  >
                    {t("common.delete")}
                  </button>
                </Show>
              </div>
              <Show when={replyOpen[comment().comment_id]}>
                <div class="mt-2">
                  <textarea
                    class={`${pageStyles.textarea} min-h-20`}
                    value={replyText[comment().comment_id] ?? ""}
                    onInput={(e) =>
                      setReplyText(comment().comment_id, e.currentTarget.value)
                    }
                    placeholder={t("blog.comments.reply_placeholder")}
                  />
                  <Show when={replyError[comment().comment_id]}>
                    <div class="text-sm text-danger">
                      {replyError[comment().comment_id]}
                    </div>
                  </Show>
                  <div class="mt-2 flex items-center gap-2">
                    <button
                      class={`${pageStyles.buttonPrimary} px-3 py-1 text-sm`}
                      disabled={
                        replyLoading[comment().comment_id] ||
                        !(replyText[comment().comment_id] ?? "").trim()
                      }
                      onClick={() => handleSubmitReply(comment().comment_id)}
                    >
                      {replyLoading[comment().comment_id]
                        ? t("blog.comments.posting")
                        : t("blog.comments.submit_reply")}
                    </button>
                    <button
                      type="button"
                      class={`${pageStyles.buttonSecondary} px-3 py-1 text-sm`}
                      onClick={() => {
                        setReplyOpen(comment().comment_id, false);
                        setReplyText(comment().comment_id, "");
                        setReplyError(comment().comment_id, null);
                      }}
                    >
                      {t("common.cancel")}
                    </button>
                  </div>
                </div>
              </Show>
              {comment().children.length > 0 &&
                renderComments(comment().children, depth + 1)}
            </div>
          );
        }}
      </Key>
    );
  }

  return (
    <main class={pageStyles.page}>
      <div class={`${pageStyles.pageInner} max-w-5xl flex flex-row gap-8`}>
        <div class="flex-1">
          <Show when={postResource.loading}>
            <div class={pageStyles.muted}>{t("blog.loading_posts")}</div>
          </Show>
          <Show when={postLoadError()}>
            <div class={pageStyles.alertError}>
              {t("blog.post.failed_load")}
              <div class="mt-2 text-sm opacity-90">{postLoadError()}</div>
              <div class="mt-3">
                <button
                  class={pageStyles.buttonSecondary}
                  onClick={() => navigate("/blog", { replace: true })}
                >
                  {t("user.back_to_blog")}
                </button>
              </div>
            </div>
          </Show>
          <Show when={postResource()}>
            {(data) => {
              const postVoteState = () =>
                optimisticVotes.post?.vote_state ?? data().vote_state;
              const postUpvotes = () =>
                optimisticVotes.post?.total_upvotes ??
                data().post.total_upvotes;
              const postDownvotes = () =>
                optimisticVotes.post?.total_downvotes ??
                data().post.total_downvotes;
              // Build the tree only when comments/local replies change; re-sort
              // separately when the sort order changes, so toggling sort does not
              // rebuild the whole tree.
              const commentTree = createMemo(() =>
                buildCommentTree(data().comments || []),
              );
              const sortedComments = createMemo(() =>
                sortCommentsTree(commentTree()),
              );
              const renderedPostHtml = createMemo(() => {
                const post = data().post;
                const clean = (raw: string) =>
                  DOMPurify.sanitize(raw, { USE_PROFILES: { html: true } });
                const content = (post.post_content ?? "").trim();
                if (content) return clean(content);
                const metadata = post.post_metadata as
                  | { markdown_content?: string }
                  | null
                  | undefined;
                const markdown =
                  typeof metadata?.markdown_content === "string"
                    ? metadata.markdown_content.trim()
                    : "";
                return markdown ? clean(markdown) : "";
              });

              return (
                <>
                  <div class="mb-4 flex flex-row items-start gap-4">
                    <div class="flex flex-col items-center pr-4 select-none border-r border-line mr-2">
                      <button
                        class={`text-2xl transition ${postVoteState() === 0 ? "text-ok font-bold" : "text-ink-muted hover:text-ok"}`}
                        onClick={() =>
                          handleVote("post", true, {
                            postId: data().post.post_id,
                          })
                        }
                        aria-label={t("blog.vote.upvote")}
                      >
                        ▲
                      </button>

                      <span class="text-sm font-semibold text-center my-1 tabular-nums text-ink">
                        {postUpvotes() - postDownvotes()}
                      </span>

                      <button
                        class={`text-2xl transition ${postVoteState() === 1 ? "text-danger font-bold" : "text-ink-muted hover:text-danger"}`}
                        onClick={() =>
                          handleVote("post", false, {
                            postId: data().post.post_id,
                          })
                        }
                        aria-label={t("blog.vote.downvote")}
                      >
                        ▼
                      </button>
                    </div>
                    <div class="flex-1">
                      <div class="flex justify-between items-start mb-2">
                        <div class="flex items-center gap-3">
                          <h1 class="text-3xl font-bold">
                            {data().post.post_title}
                          </h1>
                          <Show when={!data().post.post_is_published}>
                            <span class="rounded-full bg-accent-soft px-2 py-0.5 text-xs font-semibold uppercase tracking-wide text-accent">
                              {t("common.draft")}
                            </span>
                          </Show>
                        </div>
                        <Show
                          when={
                            user()?.user_info?.user_id &&
                            data().post.user_id === user()?.user_info?.user_id
                          }
                        >
                          <div class="flex gap-2 ml-4">
                            <button
                              class={`${pageStyles.buttonSecondary} whitespace-nowrap`}
                              onClick={() =>
                                navigate(`/blog/${data().post.post_id}/edit`)
                              }
                            >
                              {t("blog.post.edit_post")}
                            </button>
                            <button
                              class={`${pageStyles.buttonDanger} whitespace-nowrap`}
                              onClick={handleDeletePost}
                            >
                              {t("blog.post.delete_post")}
                            </button>
                          </div>
                        </Show>
                      </div>
                      <div class="flex items-center text-sm text-ink-muted mb-2 flex-wrap gap-y-1">
                        <UserBadge
                          userName={
                            data().user_badge_info?.user_name ??
                            t("common.unknown")
                          }
                          profilePictureUrl={
                            data().user_badge_info?.user_profile_picture_url
                          }
                          countryFlag={
                            data().user_badge_info?.user_country_flag
                          }
                          size="md"
                        />
                        <span class="ml-3">
                          {new Date(
                            data().post.post_created_at,
                          ).toLocaleString()}
                        </span>
                        <span class="ml-3 text-ink-faint">•</span>
                        <span>
                          {data().post.post_view_count ?? 0}{" "}
                          {t("common.views")}
                        </span>
                        <span class="ml-3 text-ink-faint">•</span>
                        <span>
                          {data().post.post_share_count ?? 0}{" "}
                          {t("common.shares")}
                        </span>
                      </div>
                      {/* Tag badges */}
                      <Show
                        when={data().post_tags && data().post_tags.length > 0}
                      >
                        <div class="flex flex-wrap gap-1.5 mb-3">
                          <For each={data().post_tags}>
                            {(tag) => (
                              <A
                                href={`/blog?q=${encodeURIComponent(tag)}&type=tag`}
                                class="inline-flex items-center px-2.5 py-1 rounded-full font-mono text-xs font-medium bg-accent-soft text-accent hover:opacity-80 transition-opacity"
                              >
                                #{tag}
                              </A>
                            )}
                          </For>
                        </div>
                      </Show>
                      <div
                        class="prose mb-3"
                        // eslint-disable-next-line solid/no-innerhtml
                        innerHTML={renderedPostHtml()}
                        ref={(el) => {
                          createEffect(() => {
                            renderedPostHtml();
                            el.querySelectorAll("pre code").forEach((block) => {
                              hljs.highlightElement(block as HTMLElement);
                            });
                          });
                        }}
                      />
                    </div>
                  </div>
                  <hr class={`my-5 ${pageStyles.divider}`} />
                  <section>
                    <div class="mb-3 flex items-center justify-between">
                      <h2 class="text-xl font-semibold">
                        {t("blog.comments.title")}
                      </h2>
                      <label class="text-sm text-ink-muted flex items-center gap-2">
                        <span>{t("blog.comments.sort_by")}</span>
                        <select
                          class={pageStyles.select}
                          value={commentSort()}
                          onChange={(e) =>
                            setCommentSort(
                              e.currentTarget.value as
                                | "best"
                                | "top"
                                | "new"
                                | "old",
                            )
                          }
                        >
                          <option value="best">{t("blog.comments.best")}</option>
                          <option value="top">{t("blog.comments.top")}</option>
                          <option value="new">{t("blog.comments.new")}</option>
                          <option value="old">{t("blog.comments.old")}</option>
                        </select>
                      </label>
                    </div>
                    {renderComments(sortedComments())}
                  </section>
                  <hr class={`my-5 ${pageStyles.divider}`} />
                  <section>
                    <h3 class="text-lg font-semibold mb-2">
                      {t("blog.comments.add")}
                    </h3>
                    <form
                      onSubmit={handleSubmitComment}
                      class="flex flex-col gap-2"
                    >
                      <textarea
                        class={pageStyles.textarea}
                        value={commentValue()}
                        onInput={(e) => setCommentValue(e.currentTarget.value)}
                        placeholder={t("blog.comments.placeholder")}
                      />
                      <Show when={commentError()}>
                        <span class="text-danger">{commentError()}</span>
                      </Show>
                      <button
                        class={`${pageStyles.buttonPrimary} self-end`}
                        type="submit"
                        disabled={commentLoading() || !commentValue().trim()}
                      >
                        {commentLoading()
                          ? t("blog.comments.posting")
                          : t("blog.comments.post")}
                      </button>
                    </form>
                  </section>
                </>
              );
            }}
          </Show>
        </div>
      </div>
    </main>
  );
}
