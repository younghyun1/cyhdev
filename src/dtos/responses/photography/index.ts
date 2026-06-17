// Photography response DTOs

export interface PhotographItem {
  photograph_id: string;
  user_id: string;
  photograph_shot_at: string | null;
  photograph_created_at: string;
  photograph_updated_at: string;
  photograph_image_type: number;
  photograph_is_on_cloud: boolean;
  photograph_link: string;
  photograph_comments: string;
  photograph_lat: number;
  photograph_lon: number;
  photograph_thumbnail_link: string;
}

export interface PaginationMeta {
  page: number;
  page_size: number;
  total_items: number;
  total_pages: number;
  has_next: boolean;
  has_prev: boolean;
}

/** Response from GET /api/photographs/get */
export interface GetPhotographsResponse {
  items: PhotographItem[];
  pagination: PaginationMeta;
}

/** Response from POST /api/photographs/upload */
export interface UploadPhotographResponse {
  photograph_id: string;
  user_id: string;
  photograph_shot_at: string | null;
  photograph_created_at: string;
  photograph_updated_at: string;
  photograph_image_type: number;
  photograph_is_on_cloud: boolean;
  photograph_link: string;
  photograph_comments: string;
  photograph_lat: number;
  photograph_lon: number;
  photograph_thumbnail_link: string;
}

/** Response from DELETE /api/photographs/delete */
export interface DeletePhotographsResponse {
  deleted_count: number;
  s3_deleted_count: number;
}

// --- Batch upload + processing tracker ---

export type BatchItemStatusName =
  | "queued"
  | "encoding"
  | "uploading"
  | "persisting"
  | "completed"
  | "failed";

/** Serde-tagged status object: `{ status, ...variant fields }`. */
export interface BatchItemStatusValue {
  status: BatchItemStatusName;
  // Present when status === "completed".
  photograph_id?: string;
  photograph_link?: string;
  thumbnail_link?: string;
  // Present when status === "failed".
  reason?: string;
}

export interface BatchItemStatus {
  item_id: string;
  file_name: string | null;
  original_size_bytes: number;
  status: BatchItemStatusValue;
  created_at: string;
  updated_at: string;
}

export interface BatchUploadItem {
  item_id: string;
  file_name: string | null;
}

/** Immediate (202) response from POST /api/photographs/batch-upload */
export interface BatchUploadResponse {
  batch_id: string;
  total: number;
  items: BatchUploadItem[];
}

/** Response from GET /api/photographs/batch/{batch_id} */
export interface BatchStatusResponse {
  batch_id: string;
  created_at: string;
  total: number;
  completed: number;
  failed: number;
  pending: number;
  done: boolean;
  items: BatchItemStatus[];
}

/** Response from GET /api/photographs/batches */
export interface BatchListResponse {
  batches: BatchStatusResponse[];
}

// --- Social: views, votes, comments ---

import type { UserBadgeInfo } from "../blog";

export type { UserBadgeInfo };

/** 0 = Upvoted, 1 = Downvoted, 2 = DidNotVote (mirrors blog VoteState). */
export type PhotographVoteState = 0 | 1 | 2;

/** The photograph row as returned by the detail endpoint (incl. counts). */
export interface PhotographDetail extends PhotographItem {
  photograph_context?: string;
  photograph_view_count: number;
  photograph_total_upvotes: number;
  photograph_total_downvotes: number;
}

export interface PhotographCommentResponse {
  photograph_comment_id: string;
  photograph_id: string;
  user_id: string;
  photograph_comment_content: string;
  photograph_comment_created_at: string;
  photograph_comment_updated_at: string | null;
  parent_photograph_comment_id: string | null;
  photograph_comment_total_upvotes: number;
  photograph_comment_total_downvotes: number;
  vote_state: PhotographVoteState;
  user_name: string;
  user_profile_picture_url: string;
  user_country_flag?: string;
}

/** Response from GET /api/photographs/{photograph_id} */
export interface ReadPhotographResponse {
  photograph: PhotographDetail;
  vote_state: PhotographVoteState;
  comments: PhotographCommentResponse[];
  user_badge_info: UserBadgeInfo;
}

/** Response from POST/DELETE photograph (comment) vote endpoints */
export interface VotePhotographResponse {
  upvote_count: number;
  downvote_count: number;
  is_upvote: boolean;
}

/** Response from DELETE /api/photographs/{photograph_id}/{comment_id} */
export interface DeletePhotographCommentResponse {
  deleted_photograph_comment_id: string;
}
