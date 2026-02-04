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
