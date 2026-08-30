// Generated from rust-be-template OpenAPI. Do not edit by hand.


export type ProcessingStatus = {
  readonly status: "queued";
} | {
  readonly status: "encoding";
} | {
  readonly status: "uploading";
} | {
  readonly status: "persisting";
} | {
  readonly photograph_id: string;
  readonly photograph_link: string;
  readonly status: "completed";
  readonly thumbnail_link: string;
} | {
  readonly reason: string;
  readonly status: "failed";
};
