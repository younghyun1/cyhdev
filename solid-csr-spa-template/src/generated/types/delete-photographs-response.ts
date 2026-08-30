// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type DeletePhotographsResponse = {
  readonly cleanup_failure_count: number;
  readonly cleanup_remaining_count: number;
  readonly deleted_count: number;
  readonly s3_deleted_count: number;
  readonly unresolved_cleanup_count: number;
};
