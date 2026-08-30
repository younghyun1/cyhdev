// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { AuthorizationAuditCursorItem } from "./authorization-audit-cursor-item";
import type { AuthorizationAuditItem } from "./authorization-audit-item";

export type AuthorizationAuditResponse = {
  readonly events: ReadonlyArray<AuthorizationAuditItem>;
  readonly next_cursor?: AuthorizationAuditCursorItem | null;
};
