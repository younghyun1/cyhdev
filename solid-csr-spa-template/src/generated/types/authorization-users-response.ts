// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { AuthorizationUserItem } from "./authorization-user-item";

export type AuthorizationUsersResponse = {
  readonly users: ReadonlyArray<AuthorizationUserItem>;
  readonly next_cursor?: string | null;
};
