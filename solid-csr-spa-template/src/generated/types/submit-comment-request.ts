// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type SubmitCommentRequest = {
  readonly comment_content: string;
  readonly guest_id?: string | null;
  readonly guest_password?: string | null;
  readonly is_guest: boolean;
  readonly parent_comment_id?: string | null;
};
