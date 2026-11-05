import { For, Show } from "solid-js";

import { locale, t } from "../../state/i18n";
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
        <table class="authorization-table authorization-audit-table">
          <thead>
            <tr>
              <th title={t("authorization.audit.time")}>{t("authorization.audit.time")}</th>
              <th title={t("authorization.audit.actor")}>{t("authorization.audit.actor")}</th>
              <th title={t("authorization.audit.change")}>{t("authorization.audit.change")}</th>
              <th title={t("authorization.audit.reason")}>{t("authorization.audit.reason")}</th>
              <th title={t("authorization.audit.request")}>{t("authorization.audit.request")}</th>
            </tr>
          </thead>
          <tbody>
            <For each={props.events}>
              {(event) => (
                <tr>
                  <td data-label={t("authorization.audit.time")}>
                    <bdi dir="auto">{new Date(event.created_at).toLocaleString(locale())}</bdi>
                  </td>
                  <td data-label={t("authorization.audit.actor")}>
                    <span dir="auto">{event.actor_display_name}</span>
                    <small><bdi dir="ltr">{event.actor_user_id}</bdi></small>
                  </td>
                  <td data-label={t("authorization.audit.change")}>
                    <span>{event.kind}</span>
                    <small>
                      <bdi dir="auto">{event.target_display_name ?? event.role_name}</bdi>:{" "}
                      <bdi dir="auto">{event.old_value}</bdi>{" "}
                      <span class="authorization-change-arrow">→</span>{" "}
                      <bdi dir="auto">{event.new_value}</bdi>
                    </small>
                  </td>
                  <td data-label={t("authorization.audit.reason")}>
                    <span class="authorization-user-text" dir="auto">{event.reason}</span>
                  </td>
                  <td data-label={t("authorization.audit.request")}>
                    <bdi dir="ltr">{event.request_id ?? t("common.n_a")}</bdi>
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
