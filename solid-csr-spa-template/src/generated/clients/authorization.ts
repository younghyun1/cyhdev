// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  AssignRoleRequest,
  AuthorizationAuditResponse,
  AuthorizationPermissionsResponse,
  AuthorizationRolesResponse,
  AuthorizationUsersResponse,
  RoleAssignmentResponse,
  RolePermissionChangeResponse,
  RolePermissionsResponse,
  SetRolePermissionRequest,
} from "../api-types";
import {
  appendQuery,
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createAuthorizationClient(transport: ApiTransport) {
  return {
    assignAuthorizationRole: async (
      input: {
        readonly path: { readonly user_id: string };
        readonly body: AssignRoleRequest;
      },
      options: ApiRequestOptions = {},
    ) => {
      const path = interpolatePath(
        "/api/admin/authorization/users/{user_id}/role",
        input.path,
      );
      return requestJson<ApiResponse<RoleAssignmentResponse>>(transport, path, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    listAuthorizationAudit: async (
      input: {
        readonly query?: {
          readonly before_created_at?: string;
          readonly before_audit_event_id?: string;
          readonly limit?: number;
        };
      } = {},
      options: ApiRequestOptions = {},
    ) => {
      const path = "/api/admin/authorization/audit";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<AuthorizationAuditResponse>>(
        transport,
        url,
        {
          method: "GET",
          headers: requestHeaders(options.headers, false),
          signal: options.signal,
        },
      );
    },
    listAuthorizationPermissions: async (
      options: ApiRequestOptions = {},
    ) => {
      const path = "/api/admin/authorization/permissions";
      return requestJson<ApiResponse<AuthorizationPermissionsResponse>>(
        transport,
        path,
        {
          method: "GET",
          headers: requestHeaders(options.headers, false),
          signal: options.signal,
        },
      );
    },
    listAuthorizationRoles: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/authorization/roles";
      return requestJson<ApiResponse<AuthorizationRolesResponse>>(
        transport,
        path,
        {
          method: "GET",
          headers: requestHeaders(options.headers, false),
          signal: options.signal,
        },
      );
    },
    listAuthorizationUsers: async (
      input: {
        readonly query?: {
          readonly search?: string;
          readonly after?: string;
          readonly limit?: number;
        };
      } = {},
      options: ApiRequestOptions = {},
    ) => {
      const path = "/api/admin/authorization/users";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<AuthorizationUsersResponse>>(
        transport,
        url,
        {
          method: "GET",
          headers: requestHeaders(options.headers, false),
          signal: options.signal,
        },
      );
    },
    listRolePermissions: async (
      input: {
        readonly query?: {
          readonly after?: string;
          readonly limit?: number;
        };
      } = {},
      options: ApiRequestOptions = {},
    ) => {
      const path = "/api/admin/authorization/role-permissions";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<RolePermissionsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    setAuthorizationRolePermission: async (
      input: {
        readonly path: {
          readonly role_id: string;
          readonly permission_id: string;
        };
        readonly body: SetRolePermissionRequest;
      },
      options: ApiRequestOptions = {},
    ) => {
      const path = interpolatePath(
        "/api/admin/authorization/roles/{role_id}/permissions/{permission_id}",
        input.path,
      );
      return requestJson<ApiResponse<RolePermissionChangeResponse>>(
        transport,
        path,
        {
          method: "PATCH",
          headers: requestHeaders(options.headers, true),
          signal: options.signal,
          body: JSON.stringify(input.body),
        },
      );
    },
  } as const;
}
