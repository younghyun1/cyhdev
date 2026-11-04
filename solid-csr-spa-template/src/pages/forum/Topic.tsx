import { For, Show, createSignal, onSettled } from "solid-js";
import { useParams } from "@solidjs/router";

import ForumReplyComposer, {
  REPLY_COMPOSER_ID,
} from "../../components/forum/ForumReplyComposer";
import ForumReplyItem from "../../components/forum/ForumReplyItem";
import ForumTopicDetailCard from "../../components/forum/ForumTopicDetail";
import { forumApi } from "../../services/contracts/forum";
import type {
  ForumCapabilitiesResponse,
  ForumReply,
  ForumReplyCursor,
  ForumReplyModerationAction,
  ForumTopic,
  ForumTopicModerationAction,
} from "../../services/contracts/forum_types";
import { user } from "../../state/auth";
import { t, tx } from "../../state/i18n";
import "../../styles/forum.css";
import { quoteForReply } from "../../utils/forumBody";
import { createReplyFragmentFollower } from "./replyFragment";

const REPLY_PAGE_SIZE = 50;
const MAX_LOCAL_REPLIES = 500;

export default function ForumTopicPage() {
  const params = useParams();
  const [topic, setTopic] = createSignal<ForumTopic | null>(null);
  const [replies, setReplies] = createSignal<ReadonlyArray<ForumReply>>([]);
  const [replyCursor, setReplyCursor] = createSignal<ForumReplyCursor | null>(null);
  const [subscribed, setSubscribed] = createSignal(false);
  const [capabilities, setCapabilities] =
    createSignal<ForumCapabilitiesResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [draft, setDraft] = createSignal("");
  let loadInFlight = false;
  const topicId = () => params.topic_id ?? "";
  const own = (authorId: string) =>
    authorId !== "00000000-0000-0000-0000-000000000000" &&
    authorId === user()?.user_info?.user_id;
  const loadTopic = async (
    cursor: ForumReplyCursor | null,
    append: boolean,
  ): Promise<void> => {
    const id = topicId();
    if (!id || loadInFlight) return;
    loadInFlight = true;
    setLoading(true);
    setError(null);
    try {
      const response = await forumApi.topic(id, cursor, REPLY_PAGE_SIZE);
      setTopic(response.data.topic);
      setSubscribed(response.data.is_subscribed);
      const countBefore = append ? replies().length : 0;
      if (append) {
        setReplies((current) => {
          const seen = new Set(current.map((reply) => reply.reply_id));
          return [
            ...current,
            ...response.data.replies.filter(
              (reply) => !seen.has(reply.reply_id),
            ),
          ].slice(0, MAX_LOCAL_REPLIES);
        });
      } else {
        setReplies(response.data.replies.slice(0, MAX_LOCAL_REPLIES));
      }
      setReplyCursor(
        countBefore + response.data.replies.length >= MAX_LOCAL_REPLIES
          ? null
          : response.data.next_reply_cursor,
      );
    } catch {
      setError(t("forum.load_failed"));
    } finally {
      loadInFlight = false;
      setLoading(false);
    }
  };
  const replyFragment = createReplyFragmentFollower({
    replies,
    cursor: replyCursor,
    error,
    loading,
    maximumReplies: MAX_LOCAL_REPLIES,
    loadPage: (nextCursor) => loadTopic(nextCursor, true),
  });
  const reload = () => {
    replyFragment.reset();
    return loadTopic(null, false);
  };
  const revisionMutation = async (
    operation: () => Promise<unknown>,
  ): Promise<void> => {
    try {
      await operation();
    } finally {
      await reload();
    }
  };
  onSettled(() => {
    void forumApi
      .capabilities()
      .then((response) => setCapabilities(response.data))
      .catch(() => setCapabilities(null));
    void reload();
  });
  const updateTopic = async (
    title: string,
    body: string,
    revision: number,
  ): Promise<void> => {
    await revisionMutation(() =>
      forumApi.updateTopic(topicId(), {
        title,
        body,
        expected_revision: revision,
      }),
    );
  };
  const deleteTopic = async (revision: number): Promise<void> => {
    await revisionMutation(() =>
      forumApi.deleteTopic(topicId(), { expected_revision: revision }),
    );
  };
  const updateSubscription = async (subscribe: boolean): Promise<void> => {
    const response = subscribe
      ? await forumApi.subscribe(topicId())
      : await forumApi.unsubscribe(topicId());
    setSubscribed(response.data.subscribed);
  };
  const createReply = async (body: string): Promise<void> => {
    const lastReply = replies().at(-1);
    await forumApi.createReply(topicId(), { body });
    if (lastReply) {
      await loadTopic(
        {
          after_reply_created_at: lastReply.created_at,
          after_reply_id: lastReply.reply_id,
        },
        true,
      );
    } else {
      await reload();
    }
    // Show the post that was just written instead of leaving the reader at an
    // emptied composer. The list renders after the signal write commits.
    requestAnimationFrame(() => {
      document
        .querySelector(".forum-replies > li:last-child")
        ?.scrollIntoView({ block: "center" });
    });
  };
  const canReply = () =>
    (capabilities()?.can_post ?? false) &&
    topic()?.access_state === "open" &&
    topic()?.content_state === "visible";
  const focusComposer = () => {
    const composer = document.getElementById(REPLY_COMPOSER_ID);
    composer?.scrollIntoView({ block: "center" });
    composer?.focus();
  };
  const quote = (name: string, body: string | null): void => {
    if (body === null) return;
    const quoted = quoteForReply(tx("forum.post.quote_attribution", { name }), body);
    setDraft((current) => (current ? `${current.trimEnd()}\n\n${quoted}` : quoted));
    requestAnimationFrame(focusComposer);
  };
  const updateReply = async (
    replyId: string,
    body: string,
    revision: number,
  ): Promise<void> => {
    await revisionMutation(() =>
      forumApi.updateReply(replyId, { body, expected_revision: revision }),
    );
  };
  const deleteReply = async (replyId: string, revision: number): Promise<void> => {
    await revisionMutation(() =>
      forumApi.deleteReply(replyId, { expected_revision: revision }),
    );
  };
  const moderateTopic = async (
    action: ForumTopicModerationAction,
    reason: string,
    revision: number,
  ): Promise<void> => {
    await revisionMutation(() =>
      forumApi.moderateTopic(topicId(), {
        action,
        reason,
        expected_revision: revision,
      }),
    );
  };
  const moderateReply = async (
    replyId: string,
    action: ForumReplyModerationAction,
    reason: string,
    revision: number,
  ): Promise<void> => {
    await revisionMutation(() =>
      forumApi.moderateReply(replyId, {
        action,
        reason,
        expected_revision: revision,
      }),
    );
  };

  return (
    <main class="forum-page">
      <div class="forum-shell">
        <nav class="forum-breadcrumb" aria-label={t("forum.breadcrumb")}>
          <a href="/forum">{t("page.forum.title")}</a>
          <span aria-hidden="true">/</span>
          <span class="forum-breadcrumb__current" aria-current="page">
            {topic() ? (topic()?.title ?? t("forum.topic.masked")) : loading() ? t("forum.loading") : ""}
          </span>
          <Show when={capabilities()?.authenticated}>
            <a class="forum-breadcrumb__end" href="/forum/notifications">
              {t("forum.notifications")}
            </a>
          </Show>
        </nav>
        <Show when={error()}>
          {(message) => (
            <div class="forum-alert forum-alert--error" role="alert">
              {message()}
            </div>
          )}
        </Show>
        <Show when={loading() && topic() === null}>
          <p class="sr-only" role="status">{t("forum.loading")}</p>
          <div class="forum-skeleton" aria-hidden="true">
            <div class="forum-skeleton__post" />
          </div>
        </Show>
        <Show when={topic()}>
          {(value) => (
            <ForumTopicDetailCard
              topic={value()}
              own={own(value().author.public_user_id)}
              authenticated={capabilities()?.authenticated ?? false}
              canModerate={capabilities()?.can_moderate ?? false}
              subscribed={subscribed()}
              onReply={canReply() ? focusComposer : undefined}
              onQuote={
                canReply()
                  ? () => quote(value().author.display_name, value().body)
                  : undefined
              }
              onUpdate={updateTopic}
              onDelete={deleteTopic}
              onSubscription={updateSubscription}
              onModerate={moderateTopic}
            />
          )}
        </Show>

        <Show when={replies().length > 0}>
          <h2 class="forum-section-title">{t("forum.reply.heading")}</h2>
        </Show>
        <ol class="forum-replies">
          <For each={replies()}>
            {(reply, index) => (
              <li>
                <ForumReplyItem
                  reply={reply}
                  number={index() + 2}
                  isOriginalPoster={
                    // Every deleted account shares one placeholder id.
                    !reply.author.is_deleted &&
                    reply.author.public_user_id === topic()?.author.public_user_id
                  }
                  own={own(reply.author.public_user_id)}
                  canModerate={capabilities()?.can_moderate ?? false}
                  onQuote={
                    canReply()
                      ? () => quote(reply.author.display_name, reply.body)
                      : undefined
                  }
                  onUpdate={(body, revision) =>
                    updateReply(reply.reply_id, body, revision)
                  }
                  onDelete={(revision) => deleteReply(reply.reply_id, revision)}
                  onModerate={(action, reason, revision) =>
                    moderateReply(
                      reply.reply_id,
                      action,
                      reason,
                      revision,
                    )
                  }
                />
              </li>
            )}
          </For>
        </ol>
        <Show when={!loading() && replies().length === 0 && topic() !== null}>
          <p class="forum-empty">{t("forum.reply.empty")}</p>
        </Show>
        <Show when={replyCursor()}>
          <div class="forum-pagination">
            <button
              class="forum-button"
              type="button"
              disabled={loading()}
              onClick={() => void loadTopic(replyCursor(), true)}
            >
              {loading() ? t("forum.loading_more") : t("forum.reply.load_more")}
            </button>
          </div>
        </Show>
        <Show when={topic() !== null}>
          <Show
            when={canReply()}
            fallback={
              <Show
                when={capabilities()?.can_post}
                fallback={
                  <Show when={capabilities() !== null && !capabilities()?.authenticated}>
                    <p class="forum-notice forum-composer-notice">
                      <a href="/login">{t("forum.sign_in")}</a>{" "}
                      <span>{t("forum.sign_in_to_reply")}</span>
                    </p>
                  </Show>
                }
              >
                <p class="forum-notice forum-composer-notice">{t("forum.reply.locked")}</p>
              </Show>
            }
          >
            <ForumReplyComposer value={draft()} onInput={setDraft} onSubmit={createReply} />
          </Show>
        </Show>
      </div>
    </main>
  );
}
