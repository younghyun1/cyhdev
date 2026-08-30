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
    assignAuthorizationRole: async (input: {
      readonly body: AssignRoleRequest;
      readonly path: {
        readonly user_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/admin/authorization/users/{user_id}/role", input.path);
      const url = path;
      return requestJson<ApiResponse<RoleAssignmentResponse>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    listAuthorizationAudit: async (input: {
      readonly query?: {
        readonly before_audit_event_id?: string;
        readonly before_created_at?: string;
        readonly limit?: number;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/admin/authorization/audit";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<AuthorizationAuditResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    listAuthorizationPermissions: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/authorization/permissions";
      const url = path;
      return requestJson<ApiResponse<AuthorizationPermissionsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    listAuthorizationRoles: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/authorization/roles";
      const url = path;
      return requestJson<ApiResponse<AuthorizationRolesResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    listAuthorizationUsers: async (input: {
      readonly query?: {
        readonly after?: string;
        readonly limit?: number;
        readonly search?: string;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/admin/authorization/users";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<AuthorizationUsersResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    listRolePermissions: async (input: {
      readonly query?: {
        readonly after?: string;
        readonly limit?: number;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/admin/authorization/role-permissions";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<RolePermissionsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    setAuthorizationRolePermission: async (input: {
      readonly body: SetRolePermissionRequest;
      readonly path: {
        readonly permission_id: string;
        readonly role_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/admin/authorization/roles/{role_id}/permissions/{permission_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<RolePermissionChangeResponse>>(transport, url, {
        method: "PATCH",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
  } as const;
}
