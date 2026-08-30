import type {
  AssignRoleRequest,
  SetRolePermissionRequest,
} from "../../generated";
import { contractApi } from "../account_api";

export type AuthorizationUsersQuery = {
  readonly search?: string;
  readonly after?: string;
  readonly limit?: number;
};

export type AuthorizationAuditQuery = {
  readonly before_created_at?: string;
  readonly before_audit_event_id?: string;
  readonly limit?: number;
};

export const authorizationAdminApi = {
  users: (query: AuthorizationUsersQuery = {}) =>
    contractApi.listAuthorizationUsers({ query }),
  roles: () => contractApi.listAuthorizationRoles(),
  permissions: () => contractApi.listAuthorizationPermissions(),
  rolePermissions: (after?: string) =>
    contractApi.listRolePermissions({ query: { after, limit: 100 } }),
  audit: (query: AuthorizationAuditQuery = {}) =>
    contractApi.listAuthorizationAudit({ query }),
  assignRole: (userId: string, body: AssignRoleRequest) =>
    contractApi.assignAuthorizationRole({
      path: { user_id: userId },
      body,
    }),
  setRolePermission: (
    roleId: string,
    permissionId: string,
    body: SetRolePermissionRequest,
  ) =>
    contractApi.setAuthorizationRolePermission({
      path: { role_id: roleId, permission_id: permissionId },
      body,
    }),
} as const;
