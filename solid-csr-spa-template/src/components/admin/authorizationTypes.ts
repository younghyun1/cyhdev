import type {
  AuthorizationAuditCursorItem,
  AuthorizationAuditItem,
  AuthorizationPermissionItem,
  AuthorizationRoleItem,
  AuthorizationUserItem,
  RolePermissionItem,
} from "../../generated";

export type AuthorizationState = {
  readonly users: readonly AuthorizationUserItem[];
  readonly roles: readonly AuthorizationRoleItem[];
  readonly permissions: readonly AuthorizationPermissionItem[];
  readonly bindings: readonly RolePermissionItem[];
  readonly auditEvents: readonly AuthorizationAuditItem[];
  readonly usersNextCursor: string | null;
  readonly auditNextCursor: AuthorizationAuditCursorItem | null;
};

export type PendingAuthorizationChange =
  | {
      readonly kind: "role";
      readonly user: AuthorizationUserItem;
      readonly role: AuthorizationRoleItem;
    }
  | {
      readonly kind: "permission";
      readonly role: AuthorizationRoleItem;
      readonly permission: AuthorizationPermissionItem;
      readonly enabled: boolean;
    };

export type {
  AuthorizationAuditCursorItem,
  AuthorizationAuditItem,
  AuthorizationPermissionItem,
  AuthorizationRoleItem,
  AuthorizationUserItem,
  RolePermissionItem,
};
