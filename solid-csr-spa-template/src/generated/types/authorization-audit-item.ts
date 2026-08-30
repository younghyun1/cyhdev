// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type AuthorizationAuditItem = {
  readonly actor_display_name: string;
  readonly actor_user_id: string;
  readonly audit_event_id: string;
  readonly created_at: string;
  readonly kind: string;
  readonly new_value: string;
  readonly old_value: string;
  readonly permission_id?: string | null;
  readonly permission_name?: string | null;
  readonly reason: string;
  readonly request_id?: string | null;
  readonly role_id: string;
  readonly role_name: string;
  readonly target_display_name?: string | null;
  readonly target_user_id?: string | null;
};
