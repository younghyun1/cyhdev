import { createMemo, For } from "solid-js";

import { t } from "../../state/i18n";
import type {
  AuthorizationPermissionItem,
  AuthorizationRoleItem,
  RolePermissionItem,
} from "./authorizationTypes";

type Props = {
  readonly roles: readonly AuthorizationRoleItem[];
  readonly permissions: readonly AuthorizationPermissionItem[];
  readonly bindings: readonly RolePermissionItem[];
  readonly onToggle: (
    role: AuthorizationRoleItem,
    permission: AuthorizationPermissionItem,
    enabled: boolean,
  ) => void;
};

export default function AuthorizationPermissionsPanel(props: Props) {
  const bindingKeys = createMemo(() =>
    new Set(
      props.bindings.map(
        (binding) => `${binding.role_id}:${binding.permission_id}`,
      ),
    ),
  );
  const enabled = (
    role: AuthorizationRoleItem,
    permission: AuthorizationPermissionItem,
  ) => bindingKeys().has(`${role.role_id}:${permission.permission_id}`);

  return (
    <section class="authorization-panel">
      <div class="authorization-panel-heading">
        <div>
          <h2>{t("authorization.permissions.title")}</h2>
          <p>{t("authorization.permissions.description")}</p>
        </div>
      </div>
      <div class="authorization-table-wrap">
        <table class="authorization-table authorization-permission-table">
          <thead>
            <tr>
              <th>{t("authorization.permission")}</th>
              <For each={props.roles}>{(role) => <th>{role.role_name}</th>}</For>
            </tr>
          </thead>
          <tbody>
            <For each={props.permissions}>
              {(permission) => (
                <tr>
                  <th scope="row">
                    <span>{permission.permission_name}</span>
                    <small>{permission.description ?? ""}</small>
                  </th>
                  <For each={props.roles}>
                    {(role) => {
                      const granted = () => enabled(role, permission);
                      return (
                        <td>
                          <button
                            type="button"
                            class={granted() ? "authorization-binding-enabled" : undefined}
                            disabled={role.role_name === "younghyun" && granted()}
                            onClick={() =>
                              props.onToggle(role, permission, !granted())
                            }
                          >
                            {granted()
                              ? t("authorization.permission.granted")
                              : t("authorization.permission.not_granted")}
                          </button>
                        </td>
                      );
                    }}
                  </For>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
    </section>
  );
}
