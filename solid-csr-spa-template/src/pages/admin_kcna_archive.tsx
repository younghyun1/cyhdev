import { createSignal, For, onSettled, Show } from "solid-js";

import AdminWorkspace from "../components/admin/AdminWorkspace";
import type { AuthorizationUserItem } from "../components/admin/authorizationTypes";
import { authorizationAdminApi } from "../services/contracts/authorization";
import {
  kcnaArchiveApi,
  type ArchiveReaderItem,
  type ArchiveReadersResponse,
} from "../services/contracts/kcna_archive";
import { locale, t } from "../state/i18n";
import { archiveAccess, setArchiveAccess } from "../state/kcna_archive_access";
import { pageStyles } from "../styles/pageStyles";
import { formatTimestamp } from "../utils/kcna_archive/format";
import { ARCHIVE_PATH } from "../utils/kcna_archive/routes";
import "../styles/kcna-archive-admin.css";

/** Matches shown for one search; the search is refined rather than paged. */
const MATCH_LIMIT = 20;

type Outcome = "saved" | "failed";

/** The accounts allowed to read the KCNA archive. Superusers always may and are not listed. */
export default function AdminKcnaArchive() {
  const [readers, setReaders] = createSignal<ReadonlyArray<ArchiveReaderItem> | null>(null);
  const [loadFailed, setLoadFailed] = createSignal(false);
  const [search, setSearch] = createSignal("");
  const [matches, setMatches] = createSignal<ReadonlyArray<AuthorizationUserItem> | null>(null);
  const [searching, setSearching] = createSignal(false);
  const [searchFailed, setSearchFailed] = createSignal(false);
  // The account whose change is in flight; one change runs at a time.
  const [changing, setChanging] = createSignal<string | null>(null);
  const [outcome, setOutcome] = createSignal<Outcome | null>(null);

  const apply = (response: ArchiveReadersResponse): void => {
    setReaders(response.readers);
    // The "administrators only" badge follows the list at once.
    const current = archiveAccess();
    if (current !== null) setArchiveAccess({ ...current, adminOnly: response.readers.length === 0 });
  };

  const load = async (): Promise<void> => {
    setLoadFailed(false);
    try {
      apply(await kcnaArchiveApi.readers());
    } catch (error) {
      console.error(error);
      setLoadFailed(true);
    }
  };

  onSettled(() => {
    void load();
  });

  const listed = (userId: string): boolean =>
    readers()?.some((reader) => reader.userId === userId) === true;

  const find = async (event: SubmitEvent): Promise<void> => {
    event.preventDefault();
    const term = search().trim();
    setSearchFailed(false);
    if (term === "") {
      setMatches(null);
      return;
    }
    setSearching(true);
    try {
      const response = await authorizationAdminApi.users({ search: term, limit: MATCH_LIMIT });
      setMatches(response.data.users);
    } catch (error) {
      console.error(error);
      setSearchFailed(true);
    } finally {
      setSearching(false);
    }
  };

  const change = async (userId: string, action: "grant" | "revoke"): Promise<void> => {
    if (changing() !== null) return;
    setChanging(userId);
    setOutcome(null);
    try {
      apply(
        action === "grant"
          ? await kcnaArchiveApi.grantReader(userId)
          : await kcnaArchiveApi.revokeReader(userId),
      );
      setOutcome("saved");
    } catch (error) {
      console.error(error);
      setOutcome("failed");
    } finally {
      setChanging(null);
    }
  };

  return (
    <AdminWorkspace>
      <div class="kcna-access">
        <header class="kcna-access-head">
          <h1 class={pageStyles.titleSm}>{t("kcna.access.title")}</h1>
          <a class={pageStyles.link} href={ARCHIVE_PATH}>
            {t("kcna.title")}
          </a>
        </header>
        <p class="kcna-access-description">{t("kcna.access.description")}</p>

        <Show when={outcome() === "saved"}>
          <p class={pageStyles.alertSuccess} role="status">
            {t("kcna.access.saved")}
          </p>
        </Show>
        <Show when={outcome() === "failed"}>
          <p class={pageStyles.alertError} role="alert">
            {t("kcna.access.failed")}
          </p>
        </Show>

        <section class="kcna-access-section" aria-labelledby="kcna-access-readers">
          <h2 id="kcna-access-readers" class="kcna-access-heading">
            {t("kcna.access.readers")}
          </h2>
          <Show when={loadFailed()}>
            <div class={pageStyles.alertError} role="alert">
              <span>{t("kcna.failure.unavailable")}</span>{" "}
              <button type="button" class={pageStyles.link} onClick={() => void load()}>
                {t("kcna.state.retry")}
              </button>
            </div>
          </Show>
          <Show when={readers()}>
            {(current) => (
              <ul class="kcna-access-list" data-list="readers">
                <For
                  each={current()}
                  fallback={<li class="kcna-access-empty">{t("kcna.access.empty")}</li>}
                >
                  {(reader) => (
                    <li class="kcna-access-row">
                      <span class="kcna-access-who">
                        <span class="kcna-access-name" dir="auto">
                          {reader.userName}
                        </span>
                        <span class="kcna-access-detail" dir="ltr">
                          {reader.userEmail}
                        </span>
                        <span class="kcna-access-detail">
                          {formatTimestamp(reader.grantedAt, locale())}
                        </span>
                      </span>
                      <button
                        type="button"
                        class="kcna-access-action"
                        disabled={changing() !== null}
                        onClick={() => void change(reader.userId, "revoke")}
                      >
                        {t("kcna.access.remove")}
                      </button>
                    </li>
                  )}
                </For>
              </ul>
            )}
          </Show>
        </section>

        <section class="kcna-access-section">
          <form class="kcna-access-search" onSubmit={(event) => void find(event)}>
            <label class="kcna-access-heading" for="kcna-access-search">
              {t("kcna.access.find")}
            </label>
            <div class="kcna-access-search-row">
              <input
                id="kcna-access-search"
                class={pageStyles.input}
                type="search"
                dir="auto"
                maxlength={100}
                value={search()}
                onInput={(event) => setSearch(event.currentTarget.value)}
              />
              <button type="submit" class="kcna-access-action" disabled={searching()}>
                {searching() ? t("common.searching") : t("kcna.search.submit")}
              </button>
            </div>
          </form>
          <Show when={searchFailed()}>
            <p class={pageStyles.alertError} role="alert">
              {t("kcna.failure.unavailable")}
            </p>
          </Show>
          <Show when={matches()}>
            {(found) => (
              <ul class="kcna-access-list" data-list="matches">
                <For
                  each={found()}
                  fallback={<li class="kcna-access-empty">{t("kcna.access.no_match")}</li>}
                >
                  {(user) => (
                    <li class="kcna-access-row">
                      <span class="kcna-access-who">
                        <span class="kcna-access-name" dir="auto">
                          {user.user_name}
                        </span>
                        <span class="kcna-access-detail" dir="ltr">
                          {user.user_email}
                        </span>
                      </span>
                      <button
                        type="button"
                        class="kcna-access-action"
                        disabled={changing() !== null || readers() === null || listed(user.user_id)}
                        onClick={() => void change(user.user_id, "grant")}
                      >
                        {t("kcna.access.allow")}
                      </button>
                    </li>
                  )}
                </For>
              </ul>
            )}
          </Show>
        </section>
      </div>
    </AdminWorkspace>
  );
}
