import { For, Show, createEffect, createMemo, createSignal, onSettled, untrack } from "solid-js";
import { useSearchParams } from "@solidjs/router";

import ForumTopicRow from "../../components/forum/ForumTopicRow";
import { forumApi } from "../../services/contracts/forum";
import type {
  ForumCapabilitiesResponse,
  ForumTopic,
  ForumTopicCursor,
} from "../../services/contracts/forum_types";
import { t, tx } from "../../state/i18n";
import "../../styles/forum.css";

const TOPIC_PAGE_SIZE = 25;
const MAX_LOCAL_TOPICS = 200;
const encoder = new TextEncoder();

const paramString = (value: string | string[] | undefined): string =>
  (Array.isArray(value) ? value[0] : value) ?? "";

function TopicGroup(props: {
  readonly title: string;
  readonly topics: ReadonlyArray<ForumTopic>;
}) {
  return (
    <section class="forum-group">
      <h2 class="forum-group__title">{props.title}</h2>
      <ol class="forum-list">
        <For each={props.topics}>
          {(topic) => (
            <li>
              <ForumTopicRow topic={topic} />
            </li>
          )}
        </For>
      </ol>
    </section>
  );
}

export default function ForumListPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [topics, setTopics] = createSignal<ReadonlyArray<ForumTopic>>([]);
  const [cursor, setCursor] = createSignal<ForumTopicCursor | null>(null);
  const [capabilities, setCapabilities] =
    createSignal<ForumCapabilitiesResponse | null>(null);
  const [searchInput, setSearchInput] = createSignal(
    untrack(() => paramString(searchParams.q)),
  );
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  // The latest request wins, so back and forward through searches cannot leave
  // an older response on screen.
  let latestRequest = 0;

  const activeSearch = () => paramString(searchParams.q).trim();
  const pinned = createMemo(() => topics().filter((topic) => topic.is_pinned));
  const latest = createMemo(() => topics().filter((topic) => !topic.is_pinned));

  const loadPage = async (
    search: string,
    nextCursor: ForumTopicCursor | null,
    replace: boolean,
  ): Promise<void> => {
    const request = ++latestRequest;
    setLoading(true);
    setError(null);
    try {
      const response = await forumApi.topics(search, nextCursor, TOPIC_PAGE_SIZE);
      if (request !== latestRequest) return;
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
      if (request === latestRequest) setError(t("forum.load_failed"));
    } finally {
      if (request === latestRequest) setLoading(false);
    }
  };

  onSettled(() => {
    void forumApi
      .capabilities()
      .then((response) => setCapabilities(response.data))
      .catch(() => setCapabilities(null));
  });

  // The URL owns the active search: it can be linked, and back restores it.
  createEffect(
    () => activeSearch(),
    (search) => {
      setSearchInput(search);
      void loadPage(search, null, true);
    },
  );

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
    if (next === activeSearch()) void loadPage(next, null, true);
    else setSearchParams({ q: next || undefined });
  };

  const showSkeleton = () => loading() && topics().length === 0;

  return (
    <main class="forum-page">
      <div class="forum-shell">
        <header class="forum-header">
          <div class="min-w-0">
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
              <a class="forum-link-button forum-button--primary" href="/forum/new">
                {t("forum.new_topic")}
              </a>
            </Show>
          </div>
        </header>

        <form class="forum-search" role="search" onSubmit={search}>
          <label for="forum-search" class="sr-only">
            {t("forum.search.label")}
          </label>
          <input
            id="forum-search"
            class="forum-input"
            type="search"
            value={searchInput()}
            placeholder={t("forum.search.placeholder")}
            onInput={(event) => setSearchInput(event.currentTarget.value)}
            maxlength={256}
          />
          <button class="forum-button forum-button--primary" type="submit">
            {t("forum.search.submit")}
          </button>
        </form>

        <Show when={capabilities() !== null && !capabilities()?.authenticated}>
          <p class="forum-notice">
            <a href="/login">{t("forum.sign_in")}</a>{" "}
            <span>{t("forum.sign_in_to_reply")}</span>
          </p>
        </Show>
        <Show when={error()}>
          {(message) => (
            <div class="forum-alert forum-alert--error" role="alert">
              {message()}
            </div>
          )}
        </Show>
        <Show when={activeSearch()}>
          <div class="forum-search-summary">
            <span>{tx("forum.search.results", { query: activeSearch() })}</span>
            <button
              class="forum-text-button"
              type="button"
              onClick={() => setSearchParams({ q: undefined })}
            >
              {t("forum.search.clear")}
            </button>
          </div>
        </Show>

        <div class="forum-board" aria-busy={loading() ? "true" : "false"}>
          <div class="forum-board__head" aria-hidden="true">
            <span>{t("forum.table.topic")}</span>
            <span>{t("forum.table.replies")}</span>
            <span>{t("forum.table.activity")}</span>
          </div>
          <Show when={showSkeleton()}>
            <p class="sr-only" role="status">{t("forum.loading")}</p>
            <div class="forum-skeleton" aria-hidden="true">
              <For each={[1, 2, 3, 4, 5]}>{() => <div class="forum-skeleton__row" />}</For>
            </div>
          </Show>
          <Show when={!loading() && topics().length === 0 && error() === null}>
            <p class="forum-empty">
              {activeSearch() ? t("forum.search.empty") : t("forum.empty")}
            </p>
          </Show>
          <Show when={pinned().length > 0}>
            <TopicGroup title={t("forum.section.pinned")} topics={pinned()} />
          </Show>
          <Show when={latest().length > 0}>
            <TopicGroup
              title={activeSearch() ? t("forum.table.topic") : t("forum.section.latest")}
              topics={latest()}
            />
          </Show>
        </div>

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
