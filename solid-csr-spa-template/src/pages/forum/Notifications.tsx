import { For, Show, createSignal, onSettled } from "solid-js";

import ForumAuthorBadge from "../../components/forum/ForumAuthor";
import { forumApi } from "../../services/contracts/forum";
import type {
  ForumNotification,
  ForumNotificationCursor,
} from "../../services/contracts/forum_types";
import { locale, t, tx } from "../../state/i18n";
import "../../styles/forum.css";

const NOTIFICATION_PAGE_SIZE = 50;
const MAX_LOCAL_NOTIFICATIONS = 200;

export default function ForumNotificationsPage() {
  const [notifications, setNotifications] = createSignal<
    ReadonlyArray<ForumNotification>
  >([]);
  const [cursor, setCursor] =
    createSignal<ForumNotificationCursor | null>(null);
  const [busyIds, setBusyIds] = createSignal<ReadonlySet<string>>(new Set());
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const readInFlight = new Set<string>();
  let loadInFlight = false;

  const load = async (
    nextCursor: ForumNotificationCursor | null,
    append: boolean,
  ): Promise<void> => {
    if (loadInFlight) return;
    loadInFlight = true;
    setLoading(true);
    setError(null);
    try {
      const response = await forumApi.notifications(
        nextCursor,
        NOTIFICATION_PAGE_SIZE,
      );
      const incoming = response.data.notifications;
      const countBefore = append ? notifications().length : 0;
      if (append) {
        setNotifications((current) => {
          const seen = new Set(
            current.map((notification) => notification.notification_id),
          );
          return [
            ...current,
            ...incoming.filter(
              (notification) => !seen.has(notification.notification_id),
            ),
          ].slice(0, MAX_LOCAL_NOTIFICATIONS);
        });
      } else {
        setNotifications(incoming.slice(0, MAX_LOCAL_NOTIFICATIONS));
      }
      setCursor(
        countBefore + incoming.length >= MAX_LOCAL_NOTIFICATIONS
          ? null
          : response.data.next_cursor,
      );
    } catch {
      setError(t("forum.notifications.load_failed"));
    } finally {
      loadInFlight = false;
      setLoading(false);
    }
  };

  onSettled(() => {
    void load(null, false);
  });

  const markRead = async (notificationId: string): Promise<void> => {
    if (readInFlight.has(notificationId)) return;
    readInFlight.add(notificationId);
    setBusyIds(new Set(readInFlight));
    try {
      const response = await forumApi.readNotification(notificationId);
      setNotifications((current) =>
        current.map((notification) =>
          notification.notification_id === notificationId
            ? { ...notification, read_at: response.data.read_at }
            : notification,
        ),
      );
    } catch {
      setError(t("forum.notifications.load_failed"));
    } finally {
      readInFlight.delete(notificationId);
      setBusyIds(new Set(readInFlight));
    }
  };

  return (
    <main class="forum-page">
      <div class="forum-shell forum-shell--narrow">
        <header class="forum-header">
          <h1 class="forum-heading">{t("page.forum.notifications_title")}</h1>
          <a class="forum-link-button" href="/forum">
            {t("forum.topic.back")}
          </a>
        </header>
        <Show when={error()}>
          {(message) => (
            <div class="forum-alert forum-alert--error" role="alert">
              {message()}
            </div>
          )}
        </Show>
        <Show when={loading() && notifications().length === 0}>
          <p class="forum-alert" role="status">{t("forum.loading")}</p>
        </Show>
        <Show when={!loading() && notifications().length === 0}>
          <p class="forum-alert">{t("forum.notifications.empty")}</p>
        </Show>
        <ol class="forum-notification-list">
          <For each={notifications()}>
            {(notification) => {
              const unread = () => notification.read_at === null;
              const actor = () => notification.actor.display_name;
              const title = () =>
                notification.topic_title ?? t("forum.topic.masked");
              return (
                <li>
                  <article
                    class={`forum-notification ${
                      unread() ? "forum-notification--unread" : ""
                    }`}
                  >
                    <div class="forum-notification__meta">
                      <ForumAuthorBadge author={notification.actor} />
                      <span>
                        {new Date(notification.created_at).toLocaleString(
                          locale(),
                        )}
                      </span>
                      <span>
                        {unread()
                          ? t("forum.notifications.unread")
                          : t("forum.notifications.read")}
                      </span>
                    </div>
                    <p class="forum-notification__body">
                      {tx("forum.notifications.topic_reply", {
                        actor: actor(),
                        topic: title(),
                      })}
                    </p>
                    <div class="forum-actions">
                      <a
                        class="forum-link-button"
                        href={`/forum/${encodeURIComponent(
                          notification.topic_id,
                        )}#forum-reply-${encodeURIComponent(
                          notification.reply_id,
                        )}`}
                      >
                        {t("forum.notifications.open_topic")}
                      </a>
                      <Show when={unread()}>
                        <button
                          class="forum-button"
                          type="button"
                          disabled={busyIds().has(notification.notification_id)}
                          onClick={() =>
                            void markRead(notification.notification_id)
                          }
                        >
                          {busyIds().has(notification.notification_id)
                            ? t("forum.notifications.marking_read")
                            : t("forum.notifications.mark_read")}
                        </button>
                      </Show>
                    </div>
                  </article>
                </li>
              );
            }}
          </For>
        </ol>
        <Show when={cursor()}>
          <div class="forum-pagination">
            <button
              class="forum-button"
              type="button"
              disabled={loading()}
              onClick={() => void load(cursor(), true)}
            >
              {loading()
                ? t("forum.loading_more")
                : t("forum.notifications.load_more")}
            </button>
          </div>
        </Show>
      </div>
    </main>
  );
}
