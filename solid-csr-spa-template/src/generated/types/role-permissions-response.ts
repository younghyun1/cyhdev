// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { RolePermissionItem } from "./role-permission-item";

export type RolePermissionsResponse = {
  readonly bindings: ReadonlyArray<RolePermissionItem>;
  readonly next_cursor?: string | null;
};
