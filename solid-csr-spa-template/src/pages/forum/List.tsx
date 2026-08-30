import { For, Show, createSignal, onSettled } from "solid-js";

import ForumTopicCard from "../../components/forum/ForumTopicCard";
import { forumApi } from "../../services/contracts/forum";
import type {
  ForumCapabilitiesResponse,
  ForumTopic,
  ForumTopicCursor,
} from "../../services/contracts/forum_types";
import { t } from "../../state/i18n";
import "../../styles/forum.css";

const TOPIC_PAGE_SIZE = 25;
const MAX_LOCAL_TOPICS = 200;
const encoder = new TextEncoder();

export default function ForumListPage() {
  const [topics, setTopics] = createSignal<ReadonlyArray<ForumTopic>>([]);
  const [cursor, setCursor] = createSignal<ForumTopicCursor | null>(null);
  const [capabilities, setCapabilities] =
    createSignal<ForumCapabilitiesResponse | null>(null);
  const [searchInput, setSearchInput] = createSignal("");
  const [activeSearch, setActiveSearch] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  let loadInFlight = false;

  const loadPage = async (
    search: string,
    nextCursor: ForumTopicCursor | null,
    replace: boolean,
  ): Promise<void> => {
    if (loadInFlight) return;
    loadInFlight = true;
    setLoading(true);
    setError(null);
    try {
      const response = await forumApi.topics(search, nextCursor, TOPIC_PAGE_SIZE);
      const incoming = response.data.topics;
      const currentCount = replace ? 0 : topics().length;
      if (replace) {
        setTopics(incoming.slice(0, MAX_LOCAL_TOPICS));
      } else {
        setTopics((current) => {
          const seen = new Set(current.map((topic) => topic.topic_id));
          const merged = [
            ...current,
            ...incoming.filter((topic) => !seen.has(topic.topic_id)),
          ];
          return merged.slice(0, MAX_LOCAL_TOPICS);
        });
      }
      const reachedLocalLimit =
        currentCount + incoming.length >= MAX_LOCAL_TOPICS;
      setCursor(reachedLocalLimit ? null : response.data.next_cursor);
    } catch {
      setError(t("forum.load_failed"));
    } finally {
      loadInFlight = false;
      setLoading(false);
    }
  };

  onSettled(() => {
    void forumApi
      .capabilities()
      .then((response) => setCapabilities(response.data))
      .catch(() => setCapabilities(null));
    void loadPage("", null, true);
  });

  const search = (event: SubmitEvent): void => {
    event.preventDefault();
    const next = searchInput().trim();
    if (
      Array.from(next).length > 128 ||
      encoder.encode(next).byteLength > 512 ||
      next.split(/\s+/u).filter(Boolean).length > 16
    ) {
      setError(t("forum.search.invalid"));
      return;
    }
    setActiveSearch(next);
    void loadPage(next, null, true);
  };

  const clearSearch = (): void => {
    setSearchInput("");
    setActiveSearch("");
    void loadPage("", null, true);
  };

  return (
    <main class="forum-page">
      <div class="forum-shell">
        <header class="forum-header">
          <div>
            <h1 class="forum-heading">{t("page.forum.title")}</h1>
            <p class="forum-subtitle">{t("forum.subtitle")}</p>
          </div>
          <div class="forum-actions">
            <Show when={capabilities()?.authenticated}>
              <a class="forum-link-button" href="/forum/notifications">
                {t("forum.notifications")}
              </a>
            </Show>
            <Show when={capabilities()?.can_post}>
              <a
                class="forum-link-button forum-button--primary"
                href="/forum/new"
              >
                {t("forum.new_topic")}
              </a>
            </Show>
          </div>
        </header>

        <form class="forum-search" role="search" onSubmit={search}>
          <label for="forum-search" class="forum-label">
            <span class="forum-help">{t("forum.search.label")}</span>
            <input
              id="forum-search"
              class="forum-input"
              value={searchInput()}
              placeholder={t("forum.search.placeholder")}
              onInput={(event) => setSearchInput(event.currentTarget.value)}
              maxlength={256}
              disabled={loading()}
            />
          </label>
          <div class="forum-actions">
            <Show when={activeSearch()}>
              <button
                class="forum-button"
                type="button"
                disabled={loading()}
                onClick={clearSearch}
              >
                {t("forum.search.clear")}
              </button>
            </Show>
            <button
              class="forum-button forum-button--primary"
              type="submit"
              disabled={loading()}
            >
              {t("forum.search.submit")}
            </button>
          </div>
        </form>

        <Show when={!capabilities()?.authenticated && capabilities() !== null}>
          <p class="forum-alert">{t("forum.sign_in_to_post")}</p>
        </Show>
        <Show when={error()}>
          {(message) => (
            <div class="forum-alert forum-alert--error" role="alert">
              {message()}
            </div>
          )}
        </Show>
        <Show when={loading() && topics().length === 0}>
          <p class="forum-alert" role="status">{t("forum.loading")}</p>
        </Show>
        <Show when={!loading() && topics().length === 0 && error() === null}>
          <p class="forum-alert">{t("forum.empty")}</p>
        </Show>
        <ol class="forum-list">
          <For each={topics()}>
            {(topic) => (
              <li>
                <ForumTopicCard topic={topic} />
              </li>
            )}
          </For>
        </ol>
        <Show when={cursor()}>
          <div class="forum-pagination">
            <button
              class="forum-button"
              type="button"
              disabled={loading()}
              onClick={() => void loadPage(activeSearch(), cursor(), false)}
            >
              {loading() ? t("forum.loading_more") : t("forum.load_more")}
            </button>
          </div>
        </Show>
      </div>
    </main>
  );
}
