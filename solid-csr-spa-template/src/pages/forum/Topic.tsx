import { For, Show, createSignal, onSettled } from "solid-js";
import { useParams } from "@solidjs/router";

import ForumReplyComposer from "../../components/forum/ForumReplyComposer";
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
import { t } from "../../state/i18n";
import "../../styles/forum.css";
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
        <div class="forum-header">
          <a class="forum-link-button" href="/forum">
            {t("forum.topic.back")}
          </a>
          <Show when={capabilities()?.authenticated}>
            <a class="forum-link-button" href="/forum/notifications">
              {t("forum.notifications")}
            </a>
          </Show>
        </div>
        <Show when={error()}>
          {(message) => (
            <div class="forum-alert forum-alert--error" role="alert">
              {message()}
            </div>
          )}
        </Show>
        <Show when={loading() && topic() === null}>
          <p class="forum-alert" role="status">{t("forum.loading")}</p>
        </Show>
        <Show when={topic()}>
          {(value) => (
            <ForumTopicDetailCard
              topic={value()}
              own={own(value().author.public_user_id)}
              authenticated={capabilities()?.authenticated ?? false}
              canModerate={capabilities()?.can_moderate ?? false}
              subscribed={subscribed()}
              onUpdate={updateTopic}
              onDelete={deleteTopic}
              onSubscription={updateSubscription}
              onModerate={moderateTopic}
            />
          )}
        </Show>

        <h2 class="forum-section-title">{t("forum.reply.heading")}</h2>
        <Show when={!loading() && replies().length === 0 && topic() !== null}>
          <p class="forum-alert">{t("forum.reply.empty")}</p>
        </Show>
        <ol class="forum-replies">
          <For each={replies()}>
            {(reply) => (
              <li>
                <ForumReplyItem
                  reply={reply}
                  own={own(reply.author.public_user_id)}
                  canModerate={capabilities()?.can_moderate ?? false}
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
        <Show when={capabilities()?.can_post && topic() !== null}>
          <ForumReplyComposer
            disabled={
              topic()?.access_state !== "open" ||
              topic()?.content_state !== "visible"
            }
            onSubmit={createReply}
          />
        </Show>
      </div>
    </main>
  );
}
