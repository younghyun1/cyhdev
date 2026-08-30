import type {
  ApiResponse,
  BatchUploadResponse,
  Photograph,
  SubmitPhotographCommentRequest,
  UpdatePhotographCommentRequest,
  VotePhotographRequest,
} from "../../generated";
import { contractApi } from "../account_api";
import { uploadWithProgress } from "../upload_with_progress";

type ProgressOptions = {
  readonly onUploadProgress?: (percent: number) => void;
};

export const photographyApi = {
  getPhotographs: (page = 1, pageSize = 20) =>
    contractApi.getPhotographs({ query: { page, page_size: pageSize } }),
  uploadPhotograph: (formData: FormData, options: ProgressOptions = {}) =>
    options.onUploadProgress
      ? uploadWithProgress<Photograph>({
          url: "/api/photographs/upload",
          formData,
          onProgress: options.onUploadProgress,
        })
      : contractApi.uploadPhotograph({ body: formData }),
  deletePhotographs: (photographIds: ReadonlyArray<string>) =>
    contractApi.deletePhotographs({
      body: { photograph_ids: photographIds },
    }),
  batchUpload: (
    formData: FormData,
    options: ProgressOptions = {},
  ): Promise<ApiResponse<BatchUploadResponse>> =>
    options.onUploadProgress
      ? uploadWithProgress<BatchUploadResponse>({
          url: "/api/photographs/batch-upload",
          formData,
          onProgress: options.onUploadProgress,
        })
      : contractApi.batchUpload({ body: formData }),
  getBatchStatus: (batchId: string) =>
    contractApi.batchStatus({ path: { batch_id: batchId } }),
  getBatches: () => contractApi.batchList(),
  getPhotographDetail: (photographId: string) =>
    contractApi.readPhotograph({ path: { photograph_id: photographId } }),
  votePhotograph: (body: VotePhotographRequest, photographId: string) =>
    contractApi.votePhotograph({
      body,
      path: { photograph_id: photographId },
    }),
  rescindPhotographVote: (photographId: string) =>
    contractApi.rescindPhotographVote({
      path: { photograph_id: photographId },
    }),
  votePhotographComment: (
    body: VotePhotographRequest,
    photographId: string,
    commentId: string,
  ) =>
    contractApi.votePhotographComment({
      body,
      path: { photograph_id: photographId, comment_id: commentId },
    }),
  rescindPhotographCommentVote: (
    photographId: string,
    commentId: string,
  ) =>
    contractApi.rescindPhotographCommentVote({
      path: { photograph_id: photographId, comment_id: commentId },
    }),
  submitPhotographComment: (
    body: SubmitPhotographCommentRequest,
    photographId: string,
  ) =>
    contractApi.submitPhotographComment({
      body,
      path: { photograph_id: photographId },
    }),
  updatePhotographComment: (
    body: UpdatePhotographCommentRequest,
    photographId: string,
    commentId: string,
  ) =>
    contractApi.updatePhotographComment({
      body,
      path: { photograph_id: photographId, comment_id: commentId },
    }),
  deletePhotographComment: (photographId: string, commentId: string) =>
    contractApi.deletePhotographComment({
      path: { photograph_id: photographId, comment_id: commentId },
    }),
} as const;
