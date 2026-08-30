// Generated from rust-be-template OpenAPI. Do not edit by hand.

export type ApiResponseMeta = {
  readonly time_to_process: string;
  readonly timestamp: string;
  readonly metadata: unknown;
};

export type ApiResponse<T> = {
  readonly success: boolean;
  readonly data: T;
  readonly meta: ApiResponseMeta;
};
