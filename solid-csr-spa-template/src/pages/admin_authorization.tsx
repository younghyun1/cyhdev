import { createSignal, onSettled, Show } from "solid-js";

import AuthorizationAuditPanel from "../components/admin/AuthorizationAuditPanel";
import AuthorizationConfirmation from "../components/admin/AuthorizationConfirmation";
import AuthorizationPermissionsPanel from "../components/admin/AuthorizationPermissionsPanel";
import AuthorizationUsersPanel from "../components/admin/AuthorizationUsersPanel";
import type {
  AuthorizationAuditCursorItem,
  AuthorizationState,
  PendingAuthorizationChange,
  RolePermissionItem,
} from "../components/admin/authorizationTypes";
import { authorizationAdminApi } from "../services/contracts/authorization";
import { t } from "../state/i18n";
import "../styles/authorization.css";

const PAGE_SIZE = 50;
const MAX_BINDING_PAGES = 11;

const EMPTY_STATE: AuthorizationState = {
  users: [],
  roles: [],
  permissions: [],
  bindings: [],
  auditEvents: [],
  usersNextCursor: null,
  auditNextCursor: null,
};

export default function AdminAuthorizationPage() {
  const [state, setState] = createSignal<AuthorizationState>(EMPTY_STATE);
  const [activeSearch, setActiveSearch] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [pageError, setPageError] = createSignal<string | null>(null);
  const [pending, setPending] =
    createSignal<PendingAuthorizationChange | null>(null);
  const [mutationBusy, setMutationBusy] = createSignal(false);
  const [mutationError, setMutationError] = createSignal<string | null>(null);

  onSettled(() => {
    void loadInitial();
  });

  const loadInitial = async () => {
    setLoading(true);
    setPageError(null);
    try {
      const [users, roles, permissions, bindings, audit] = await Promise.all([
        authorizationAdminApi.users({ limit: PAGE_SIZE }),
        authorizationAdminApi.roles(),
        authorizationAdminApi.permissions(),
        loadAllRolePermissions(),
        authorizationAdminApi.audit({ limit: PAGE_SIZE }),
      ]);
      setState({
        users: users.data.users,
        roles: roles.data.roles,
        permissions: permissions.data.permissions,
        bindings,
        auditEvents: audit.data.events,
        usersNextCursor: users.data.next_cursor ?? null,
        auditNextCursor: audit.data.next_cursor ?? null,
      });
    } catch (error: unknown) {
      setPageError(errorMessage(error));
    } finally {
      setLoading(false);
    }
  };

  const searchUsers = async (search: string) => {
    setLoading(true);
    setPageError(null);
    setActiveSearch(search);
    try {
      const response = await authorizationAdminApi.users({
        search: search || undefined,
        limit: PAGE_SIZE,
      });
      setState((current) => ({
        ...current,
        users: response.data.users,
        usersNextCursor: response.data.next_cursor ?? null,
      }));
    } catch (error: unknown) {
      setPageError(errorMessage(error));
    } finally {
      setLoading(false);
    }
  };

  const nextUsers = async () => {
    const cursor = state().usersNextCursor;
    if (cursor === null) return;
    setLoading(true);
    try {
      const response = await authorizationAdminApi.users({
        search: activeSearch() || undefined,
        after: cursor,
        limit: PAGE_SIZE,
      });
      setState((current) => ({
        ...current,
        users: response.data.users,
        usersNextCursor: response.data.next_cursor ?? null,
      }));
    } catch (error: unknown) {
      setPageError(errorMessage(error));
    } finally {
      setLoading(false);
    }
  };

  const nextAudit = async () => {
    const cursor = state().auditNextCursor;
    if (cursor === null) return;
    setLoading(true);
    try {
      const response = await authorizationAdminApi.audit(auditQuery(cursor));
      setState((current) => ({
        ...current,
        auditEvents: response.data.events,
        auditNextCursor: response.data.next_cursor ?? null,
      }));
    } catch (error: unknown) {
      setPageError(errorMessage(error));
    } finally {
      setLoading(false);
    }
  };

  const applyChange = async (reason: string) => {
    const change = pending();
    if (change === null) return;
    setMutationBusy(true);
    setMutationError(null);
    try {
      if (change.kind === "role") {
        await authorizationAdminApi.assignRole(change.user.user_id, {
          role_id: change.role.role_id,
          reason,
          confirmed: true,
          confirmed_user_id: change.user.user_id,
        });
      } else {
        await authorizationAdminApi.setRolePermission(
          change.role.role_id,
          change.permission.permission_id,
          {
            enabled: change.enabled,
            reason,
            confirmed: true,
            confirmed_role_id: change.role.role_id,
            confirmed_permission_id: change.permission.permission_id,
          },
        );
      }
      await refreshAfterMutation(change.kind);
      setPending(null);
    } catch (error: unknown) {
      setMutationError(errorMessage(error));
    } finally {
      setMutationBusy(false);
    }
  };

  const refreshAfterMutation = async (kind: PendingAuthorizationChange["kind"]) => {
    const usersPromise = authorizationAdminApi.users({
      search: activeSearch() || undefined,
      limit: PAGE_SIZE,
    });
    const auditPromise = authorizationAdminApi.audit({ limit: PAGE_SIZE });
    const bindingsPromise =
      kind === "permission"
        ? loadAllRolePermissions()
        : Promise.resolve(state().bindings);
    const [users, audit, bindings] = await Promise.all([
      usersPromise,
      auditPromise,
      bindingsPromise,
    ]);
    setState((current) => ({
      ...current,
      users: users.data.users,
      usersNextCursor: users.data.next_cursor ?? null,
      auditEvents: audit.data.events,
      auditNextCursor: audit.data.next_cursor ?? null,
      bindings,
    }));
  };

  return (
    <main class="authorization-page">
      <div class="authorization-page-inner">
        <header class="authorization-page-header">
          <div>
            <p class="authorization-eyebrow">{t("authorization.eyebrow")}</p>
            <h1>{t("page.authorization.title")}</h1>
            <p>{t("authorization.subtitle")}</p>
          </div>
          <button type="button" onClick={loadInitial} disabled={loading()}>
            {t("common.refresh")}
          </button>
        </header>
        <Show when={pageError() !== null}>
          <p class="authorization-error" role="alert">{pageError()}</p>
        </Show>
        <Show when={!loading() || state().roles.length > 0} fallback={
          <p class="authorization-loading">{t("common.loading")}</p>
        }>
          <AuthorizationUsersPanel
            users={state().users}
            roles={state().roles}
            nextCursor={state().usersNextCursor}
            loading={loading()}
            onSearch={searchUsers}
            onNext={nextUsers}
            onAssign={(user, role) => {
              setMutationError(null);
              setPending({ kind: "role", user, role });
            }}
          />
          <AuthorizationPermissionsPanel
            roles={state().roles}
            permissions={state().permissions}
            bindings={state().bindings}
            onToggle={(role, permission, enabled) => {
              setMutationError(null);
              setPending({ kind: "permission", role, permission, enabled });
            }}
          />
          <AuthorizationAuditPanel
            events={state().auditEvents}
            nextCursor={state().auditNextCursor}
            loading={loading()}
            onNext={nextAudit}
          />
        </Show>
      </div>
      <AuthorizationConfirmation
        change={pending()}
        busy={mutationBusy()}
        error={mutationError()}
        onCancel={() => {
          if (!mutationBusy()) setPending(null);
        }}
        onConfirm={applyChange}
      />
    </main>
  );
}

async function loadAllRolePermissions(): Promise<readonly RolePermissionItem[]> {
  const bindings: RolePermissionItem[] = [];
  let after: string | undefined;
  for (let page = 0; page < MAX_BINDING_PAGES; page += 1) {
    const response = await authorizationAdminApi.rolePermissions(after);
    bindings.push(...response.data.bindings);
    const next = response.data.next_cursor;
    if (next === undefined || next === null) return bindings;
    after = next;
  }
  throw new Error("Role-permission catalog exceeded its fixed client bound.");
}

function auditQuery(cursor: AuthorizationAuditCursorItem) {
  return {
    before_created_at: cursor.created_at,
    before_audit_event_id: cursor.audit_event_id,
    limit: PAGE_SIZE,
  } as const;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : t("authorization.error.unknown");
}
