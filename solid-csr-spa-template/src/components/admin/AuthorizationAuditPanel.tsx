import { For, Show } from "solid-js";

import { t } from "../../state/i18n";
import type {
  AuthorizationAuditCursorItem,
  AuthorizationAuditItem,
} from "./authorizationTypes";

type Props = {
  readonly events: readonly AuthorizationAuditItem[];
  readonly nextCursor: AuthorizationAuditCursorItem | null;
  readonly loading: boolean;
  readonly onNext: () => Promise<void>;
};

export default function AuthorizationAuditPanel(props: Props) {
  return (
    <section class="authorization-panel">
      <div class="authorization-panel-heading">
        <div>
          <h2>{t("authorization.audit.title")}</h2>
          <p>{t("authorization.audit.description")}</p>
        </div>
      </div>
      <div class="authorization-table-wrap">
        <table class="authorization-table">
          <thead>
            <tr>
              <th>{t("authorization.audit.time")}</th>
              <th>{t("authorization.audit.actor")}</th>
              <th>{t("authorization.audit.change")}</th>
              <th>{t("authorization.audit.reason")}</th>
              <th>{t("authorization.audit.request")}</th>
            </tr>
          </thead>
          <tbody>
            <For each={props.events}>
              {(event) => (
                <tr>
                  <td data-label={t("authorization.audit.time")}>
                    {new Date(event.created_at).toLocaleString()}
                  </td>
                  <td data-label={t("authorization.audit.actor")}>
                    <span>{event.actor_display_name}</span>
                    <small>{event.actor_user_id}</small>
                  </td>
                  <td data-label={t("authorization.audit.change")}>
                    <span>{event.kind}</span>
                    <small>
                      {event.target_display_name ?? event.role_name}: {event.old_value} →{" "}
                      {event.new_value}
                    </small>
                  </td>
                  <td data-label={t("authorization.audit.reason")}>
                    {event.reason}
                  </td>
                  <td data-label={t("authorization.audit.request")}>
                    {event.request_id ?? t("common.n_a")}
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
      <Show when={props.events.length === 0}>
        <p class="authorization-empty">{t("authorization.audit.empty")}</p>
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
