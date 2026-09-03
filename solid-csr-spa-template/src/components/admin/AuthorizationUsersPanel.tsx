import { createSignal, For, Show } from "solid-js";

import { t } from "../../state/i18n";
import type {
  AuthorizationRoleItem,
  AuthorizationUserItem,
} from "./authorizationTypes";

type Props = {
  readonly users: readonly AuthorizationUserItem[];
  readonly roles: readonly AuthorizationRoleItem[];
  readonly nextCursor: string | null;
  readonly loading: boolean;
  readonly onSearch: (search: string) => Promise<void>;
  readonly onNext: () => Promise<void>;
  readonly onAssign: (
    user: AuthorizationUserItem,
    role: AuthorizationRoleItem,
  ) => void;
};

export default function AuthorizationUsersPanel(props: Props) {
  const [search, setSearch] = createSignal("");
  const [selectedRoles, setSelectedRoles] = createSignal<
    Readonly<Record<string, string>>
  >({});

  const selectedRoleId = (user: AuthorizationUserItem) =>
    selectedRoles()[user.user_id] ?? user.role_id;

  const submitSearch = async (event: SubmitEvent) => {
    event.preventDefault();
    await props.onSearch(search().trim());
  };

  const assign = (user: AuthorizationUserItem) => {
    const role = props.roles.find(
      (candidate) => candidate.role_id === selectedRoleId(user),
    );
    if (role !== undefined && role.role_id !== user.role_id) {
      props.onAssign(user, role);
    }
  };

  return (
    <section class="authorization-panel">
      <div class="authorization-panel-heading">
        <div>
          <h2>{t("authorization.users.title")}</h2>
          <p>{t("authorization.users.description")}</p>
        </div>
        <form class="authorization-search" onSubmit={submitSearch}>
          <label for="authorization-user-search" class="authorization-sr-only">
            {t("authorization.users.search")}
          </label>
          <input
            id="authorization-user-search"
            type="search"
            maxlength={100}
            value={search()}
            placeholder={t("authorization.users.search")}
            onInput={(event) => setSearch(event.currentTarget.value)}
          />
          <button type="submit" disabled={props.loading}>
            {t("common.lookup")}
          </button>
        </form>
      </div>
      <div class="authorization-table-wrap">
        <table class="authorization-table">
          <thead>
            <tr>
              <th>{t("common.username")}</th>
              <th>{t("common.email")}</th>
              <th>{t("authorization.role")}</th>
              <th>{t("authorization.action")}</th>
            </tr>
          </thead>
          <tbody>
            <For each={props.users}>
              {(user) => (
                <tr>
                  <td data-label={t("common.username")}>{user.user_name}</td>
                  <td data-label={t("common.email")}>{user.user_email}</td>
                  <td data-label={t("authorization.role")}>
                    <select
                      aria-label={`${t("authorization.role")}: ${user.user_name}`}
                      value={selectedRoleId(user)}
                      onChange={(event) =>
                        setSelectedRoles((current) => ({
                          ...current,
                          [user.user_id]: event.currentTarget.value,
                        }))
                      }
                    >
                      <For each={props.roles}>
                        {(role) => (
                          <option value={role.role_id}>{role.role_name}</option>
                        )}
                      </For>
                    </select>
                  </td>
                  <td data-label={t("authorization.action")}>
                    <button
                      type="button"
                      disabled={selectedRoleId(user) === user.role_id}
                      onClick={() => assign(user)}
                    >
                      {t("authorization.role.assign")}
                    </button>
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
      <Show when={props.users.length === 0}>
        <p class="authorization-empty">{t("authorization.users.empty")}</p>
      </Show>
      <Show when={props.nextCursor !== null}>
        <button
          type="button"
          disabled={props.loading}
          onClick={() => props.onNext()}
        >
          {t("common.next")}
        </button>
      </Show>
    </section>
  );
}
