// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type SubmitPostRequest = {
  readonly post_content: string;
  readonly post_id?: string | null;
  readonly post_is_published: boolean;
  readonly post_tags: ReadonlyArray<string>;
  readonly post_title: string;
};
