// Social block for the photograph detail modal: view count, vote control,
// threaded comments, composer, and author/superuser edit/delete. Mirrors the
// blog `posts/View.tsx` visual + interaction patterns (emerald up / rose down,
// optimistic votes with rollback) against the separate photograph endpoints.
//
// The detail resource is fetched once per photograph (it increments the naive
// view count server-side), so comment mutations update a LOCAL comment list
// instead of refetching — otherwise every comment action would re-inflate the
// view count.

import {
  Loading,
  Show,
  createEffect,
  createMemo,
  createSignal,
  createStore,
} from "solid-js";
import { Key } from "@solid-primitives/keyed";
import { useNavigate } from "@solidjs/router";
import { photographyApi } from "../../services/all_api";
import { createKeyedStore } from "../../state/keyed_store";
import { isSuperuser, user } from "../../state/auth";
import { pageStyles } from "../../styles/pageStyles";
import { t } from "../../state/i18n";
import { UserBadge } from "../UserBadge";
import type { PhotographCommentResponse } from "../../dtos/responses/photography";

type VoteState = 0 | 1 | 2;

const COMMENT_INDENT_PX = 16;

interface CommentNode extends PhotographCommentResponse {
  children: CommentNode[];
}

interface OptimisticVote {
  up: number;
  down: number;
  vs: VoteState;
}

interface PhotographSocialProps {
  photographId: string;
}

export default function PhotographSocial(props: PhotographSocialProps) {
  const navigate = useNavigate();

  // Fetched once per photographId (this GET increments the view count).
  const detail = createMemo(async () => {
    const id = props.photographId;
    return (await photographyApi.getPhotographDetail(id)).data;
  });

  // Local, mutable comment list seeded from the detail load. Comment mutations
  // edit this list directly so we never refetch (which would re-count a view).
  const [comments, setComments] = createSignal<PhotographCommentResponse[]>([]);
  createEffect(
    () => detail(),
    (d) => {
      setComments(d.comments);
    },
  );

  const [optimistic, setOptimistic] = createStore<{
    photo?: OptimisticVote;
    comments: Record<string, OptimisticVote>;
  }>({ comments: {} });

  const [commentValue, setCommentValue] = createSignal("");
  const [commentBusy, setCommentBusy] = createSignal(false);

  const [replyOpen, setReplyOpen] = createKeyedStore<boolean>();
  const [replyText, setReplyText] = createKeyedStore<string>();
  const [editOpen, setEditOpen] = createKeyedStore<boolean>();
  const [editText, setEditText] = createKeyedStore<string>();
  const [rowBusy, setRowBusy] = createKeyedStore<boolean>();

  const meId = () => user()?.user_info?.user_id;
  const canModify = (authorId: string) =>
    !!meId() && (authorId === meId() || isSuperuser() === true);

  const commentTree = createMemo<CommentNode[]>(() => {
    const list = comments();
    const map = new Map<string, CommentNode>();
    for (const c of list)
      map.set(c.photograph_comment_id, { ...c, children: [] });
    const roots: CommentNode[] = [];
    for (const c of list) {
      const node = map.get(c.photograph_comment_id)!;
      const parentId = c.parent_photograph_comment_id;
      if (parentId && map.has(parentId)) {
        map.get(parentId)!.children.push(node);
      } else {
        roots.push(node);
      }
    }
    const netScore = (n: CommentNode) =>
      n.photograph_comment_total_upvotes - n.photograph_comment_total_downvotes;
    const sortNodes = (nodes: CommentNode[]) => {
      nodes.sort((a, b) => netScore(b) - netScore(a));
      for (const n of nodes) sortNodes(n.children);
    };
    sortNodes(roots);
    return roots;
  });

  // --- Voting (optimistic, with rollback) ---
  const photoVote = createMemo<OptimisticVote>(() => {
    if (optimistic.photo) return optimistic.photo;
    const d = detail();
    return {
      up: d.photograph.photograph_total_upvotes,
      down: d.photograph.photograph_total_downvotes,
      vs: d.vote_state,
    };
  });

  const commentVote = (c: PhotographCommentResponse): OptimisticVote =>
    optimistic.comments[c.photograph_comment_id] ?? {
      up: c.photograph_comment_total_upvotes,
      down: c.photograph_comment_total_downvotes,
      vs: c.vote_state,
    };

  const applyVote = (
    current: OptimisticVote,
    isUpvote: boolean,
  ): OptimisticVote => {
    const rescinding =
      (isUpvote && current.vs === 0) || (!isUpvote && current.vs === 1);
    let up = current.up;
    let down = current.down;
    let vs: VoteState;
    if (rescinding) {
      vs = 2;
      if (isUpvote) up -= 1;
      else down -= 1;
    } else {
      if (current.vs === 0) up -= 1;
      if (current.vs === 1) down -= 1;
      if (isUpvote) up += 1;
      else down += 1;
      vs = isUpvote ? 0 : 1;
    }
    return { up, down, vs };
  };

  const votePhoto = async (isUpvote: boolean) => {
    if (!meId()) {
      navigate("/login");
      return;
    }
    const current = photoVote();
    const rescinding =
      (isUpvote && current.vs === 0) || (!isUpvote && current.vs === 1);
    const next = applyVote(current, isUpvote);
    setOptimistic((s) => {
      s.photo = next;
    });
    try {
      if (rescinding) {
        await photographyApi.rescindPhotographVote(props.photographId);
      } else {
        await photographyApi.votePhotograph(
          { is_upvote: isUpvote },
          props.photographId,
        );
      }
    } catch (err) {
      console.error("Photo vote failed:", err);
      setOptimistic((s) => {
        s.photo = current;
      });
    }
  };

  const voteComment = async (
    comment: PhotographCommentResponse,
    isUpvote: boolean,
  ) => {
    if (!meId()) {
      navigate("/login");
      return;
    }
    const id = comment.photograph_comment_id;
    const current = commentVote(comment);
    const rescinding =
      (isUpvote && current.vs === 0) || (!isUpvote && current.vs === 1);
    const next = applyVote(current, isUpvote);
    setOptimistic((s) => {
      s.comments[id] = next;
    });
    try {
      if (rescinding) {
        await photographyApi.rescindPhotographCommentVote(
          props.photographId,
          id,
        );
      } else {
        await photographyApi.votePhotographComment(
          { is_upvote: isUpvote },
          props.photographId,
          id,
        );
      }
    } catch (err) {
      console.error("Comment vote failed:", err);
      setOptimistic((s) => {
        s.comments[id] = current;
      });
    }
  };

  // --- Comment mutations (local list updates, no refetch) ---
  const submitTopComment = async (e: Event) => {
    e.preventDefault();
    const content = commentValue().trim();
    if (!content) return;
    setCommentBusy(true);
    try {
      const created = await photographyApi.submitPhotographComment(
        { parent_comment_id: null, comment_content: content },
        props.photographId,
      );
      setComments((prev) => [...prev, created.data]);
      setCommentValue("");
    } catch (err) {
      console.error("Comment failed:", err);
    } finally {
      setCommentBusy(false);
    }
  };

  const submitReply = async (parentId: string) => {
    const content = (replyText[parentId] ?? "").trim();
    if (!content) return;
    setRowBusy(parentId, true);
    try {
      const created = await photographyApi.submitPhotographComment(
        { parent_comment_id: parentId, comment_content: content },
        props.photographId,
      );
      setComments((prev) => [...prev, created.data]);
      setReplyText(parentId, "");
      setReplyOpen(parentId, false);
    } catch (err) {
      console.error("Reply failed:", err);
    } finally {
      setRowBusy(parentId, false);
    }
  };

  const saveEdit = async (commentId: string) => {
    const content = (editText[commentId] ?? "").trim();
    if (!content) return;
    setRowBusy(commentId, true);
    try {
      const updated = await photographyApi.updatePhotographComment(
        { comment_content: content },
        props.photographId,
        commentId,
      );
      setComments((prev) =>
        prev.map((c) =>
          c.photograph_comment_id === commentId ? updated.data : c,
        ),
      );
      setEditOpen(commentId, false);
    } catch (err) {
      console.error("Edit failed:", err);
    } finally {
      setRowBusy(commentId, false);
    }
  };

  // Remove a comment and all of its descendants (server cascade-deletes them).
  const removeSubtree = (
    list: PhotographCommentResponse[],
    rootId: string,
  ): PhotographCommentResponse[] => {
    const doomed = new Set<string>([rootId]);
    let changed = true;
    while (changed) {
      changed = false;
      for (const c of list) {
        const parent = c.parent_photograph_comment_id;
        if (
          parent &&
          doomed.has(parent) &&
          !doomed.has(c.photograph_comment_id)
        ) {
          doomed.add(c.photograph_comment_id);
          changed = true;
        }
      }
    }
    return list.filter((c) => !doomed.has(c.photograph_comment_id));
  };

  const removeComment = async (commentId: string) => {
    if (!confirm(t("photos.delete_confirm").replace("{count}", "1"))) return;
    setRowBusy(commentId, true);
    try {
      await photographyApi.deletePhotographComment(
        props.photographId,
        commentId,
      );
      setComments((prev) => removeSubtree(prev, commentId));
    } catch (err) {
      console.error("Delete comment failed:", err);
    } finally {
      setRowBusy(commentId, false);
    }
  };

  const renderComments = (nodes: CommentNode[], depth = 0) => (
    <Key each={nodes} by={(comment) => comment.photograph_comment_id}>
      {(comment) => {
        const cv = () => commentVote(comment());
        const id = () => comment().photograph_comment_id;
        return (
          <div
            class="mt-2 pl-3 border-l border-line"
            style={{ "margin-left": `${depth * COMMENT_INDENT_PX}px` }}
          >
            <div class="mb-1 flex flex-wrap items-center gap-2 text-xs text-ink-muted">
              <UserBadge
                userName={comment().user_name || t("common.unknown")}
                profilePictureUrl={comment().user_profile_picture_url}
                countryFlag={comment().user_country_flag}
                size="sm"
              />
              <span class="ml-2">
                {new Date(
                  comment().photograph_comment_created_at,
                ).toLocaleString()}
              </span>
            </div>

            <Show
              when={editOpen[id()]}
              fallback={
                <div class="text-ink whitespace-pre-wrap text-sm">
                  {comment().photograph_comment_content}
                </div>
              }
            >
              <div class="mt-1">
                <textarea
                  class={`${pageStyles.textarea} min-h-20`}
                  value={editText[id()] ?? ""}
                  onInput={(e) => setEditText(id(), e.currentTarget.value)}
                />
                <div class="mt-1 flex gap-2">
                  <button
                    class={`${pageStyles.buttonPrimary} px-3 py-1 text-sm`}
                    disabled={rowBusy[id()] || !(editText[id()] ?? "").trim()}
                    onClick={() => saveEdit(id())}
                  >
                    {rowBusy[id()] ? t("common.saving") : t("common.save")}
                  </button>
                  <button
                    class={`${pageStyles.buttonSecondary} px-3 py-1 text-sm`}
                    onClick={() => setEditOpen(id(), false)}
                  >
                    {t("common.cancel")}
                  </button>
                </div>
              </div>
            </Show>

            <div class="flex items-center gap-2 mt-1">
              <button
                class={`text-lg px-1 ${cv().vs === 0 ? "text-ok font-bold" : "text-ink-muted hover:text-ok"}`}
                onClick={() => voteComment(comment(), true)}
                title={t("blog.vote.upvote")}
              >
                ▲
              </button>
              <span class="text-xs font-semibold text-ink">
                {cv().up - cv().down}
              </span>
              <button
                class={`text-lg px-1 ${cv().vs === 1 ? "text-danger font-bold" : "text-ink-muted hover:text-danger"}`}
                onClick={() => voteComment(comment(), false)}
                title={t("blog.vote.downvote")}
              >
                ▼
              </button>
            </div>

            <div class="mt-1 flex gap-3">
              <Show when={meId()}>
                <button
                  class={`${pageStyles.link} text-xs`}
                  onClick={() => setReplyOpen(id(), !replyOpen[id()])}
                >
                  {t("blog.comments.reply")}
                </button>
              </Show>
              <Show when={canModify(comment().user_id)}>
                <button
                  class={`${pageStyles.link} text-xs`}
                  onClick={() => {
                    setEditText(id(), comment().photograph_comment_content);
                    setEditOpen(id(), true);
                  }}
                >
                  {t("common.edit")}
                </button>
                <button
                  class="text-xs text-danger hover:underline"
                  onClick={() => removeComment(id())}
                >
                  {t("common.delete")}
                </button>
              </Show>
            </div>

            <Show when={replyOpen[id()]}>
              <div class="mt-2">
                <textarea
                  class={`${pageStyles.textarea} min-h-20`}
                  value={replyText[id()] ?? ""}
                  onInput={(e) => setReplyText(id(), e.currentTarget.value)}
                  placeholder={t("blog.comments.reply_placeholder")}
                />
                <div class="mt-1 flex gap-2">
                  <button
                    class={`${pageStyles.buttonPrimary} px-3 py-1 text-sm`}
                    disabled={rowBusy[id()] || !(replyText[id()] ?? "").trim()}
                    onClick={() => submitReply(id())}
                  >
                    {rowBusy[id()]
                      ? t("blog.comments.posting")
                      : t("blog.comments.submit_reply")}
                  </button>
                  <button
                    class={`${pageStyles.buttonSecondary} px-3 py-1 text-sm`}
                    onClick={() => setReplyOpen(id(), false)}
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

  return (
    <div class="flex flex-col gap-4">
      <Loading>
        {/* Views + photograph vote */}
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <button
              class={`text-2xl transition ${photoVote().vs === 0 ? "text-ok font-bold" : "text-ink-muted hover:text-ok"}`}
              onClick={() => votePhoto(true)}
              aria-label={t("blog.vote.upvote")}
            >
              ▲
            </button>
            <span class="text-sm font-semibold text-ink">
              {photoVote().up - photoVote().down}
            </span>
            <button
              class={`text-2xl transition ${photoVote().vs === 1 ? "text-danger font-bold" : "text-ink-muted hover:text-danger"}`}
              onClick={() => votePhoto(false)}
              aria-label={t("blog.vote.downvote")}
            >
              ▼
            </button>
          </div>
          <span class="text-sm text-ink-muted">
            {detail().photograph.photograph_view_count} {t("common.views")}
          </span>
        </div>

        {/* Comment composer */}
        <Show when={meId()}>
          <form onSubmit={submitTopComment} class="flex flex-col gap-2">
            <h3 class="text-sm font-bold text-ink-muted uppercase tracking-wide">
              {t("blog.comments.add")}
            </h3>
            <textarea
              class={pageStyles.textarea}
              value={commentValue()}
              onInput={(e) => setCommentValue(e.currentTarget.value)}
              placeholder={t("blog.comments.placeholder")}
            />
            <button
              class={`${pageStyles.buttonPrimary} self-end`}
              type="submit"
              disabled={commentBusy() || !commentValue().trim()}
            >
              {commentBusy()
                ? t("blog.comments.posting")
                : t("blog.comments.post")}
            </button>
          </form>
        </Show>

        {/* Comment list */}
        <div>
          <Show
            when={commentTree().length > 0}
            fallback={
              <p class="text-sm text-ink-muted">
                {t("photos.no_comments")}
              </p>
            }
          >
            {renderComments(commentTree())}
          </Show>
        </div>
      </Loading>
    </div>
  );
}
