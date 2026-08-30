import type {
  SubmitCommentRequest,
  SubmitPostRequest,
  UpdateCommentRequest,
  UpdatePostRequest,
  UpvoteCommentRequest,
  UpvotePostRequest,
} from "../../generated";
import { contractApi } from "../account_api";

type GetPostsInput = NonNullable<Parameters<typeof contractApi.getPosts>[0]>;
export type GetPostsRequest = NonNullable<GetPostsInput["query"]>;

export const blogApi = {
  getPosts: (query: GetPostsRequest = {}) =>
    contractApi.getPosts({ query }),
  readPost: (postId: string) =>
    contractApi.readPost({ path: { post_id: postId } }),
  submitPost: (body: SubmitPostRequest) => contractApi.submitPost({ body }),
  updatePost: (body: UpdatePostRequest, postId: string) =>
    contractApi.updatePost({ body, path: { post_id: postId } }),
  votePost: (body: UpvotePostRequest, postId: string) =>
    contractApi.votePost({ body, path: { post_id: postId } }),
  voteComment: (
    body: UpvoteCommentRequest,
    postId: string,
    commentId: string,
  ) =>
    contractApi.voteComment({
      body,
      path: { post_id: postId, comment_id: commentId },
    }),
  rescindPostVote: (postId: string) =>
    contractApi.rescindPostVote({ path: { post_id: postId } }),
  rescindCommentVote: (postId: string, commentId: string) =>
    contractApi.rescindCommentVote({
      path: { post_id: postId, comment_id: commentId },
    }),
  submitComment: (body: SubmitCommentRequest, postId: string) =>
    contractApi.submitComment({ body, path: { post_id: postId } }),
  updateComment: (
    body: UpdateCommentRequest,
    postId: string,
    commentId: string,
  ) =>
    contractApi.updateComment({
      body,
      path: { post_id: postId, comment_id: commentId },
    }),
  deletePost: (postId: string) =>
    contractApi.deletePost({ path: { post_id: postId } }),
  deleteComment: (postId: string, commentId: string) =>
    contractApi.deleteComment({
      path: { post_id: postId, comment_id: commentId },
    }),
  searchPosts: (
    query: string,
    searchType: "title" | "tag" = "title",
    page = 1,
    limit = 20,
    tags?: ReadonlyArray<string>,
  ) =>
    contractApi.searchPosts({
      query: {
        q: query,
        search_type: searchType,
        page,
        limit,
        tags: tags && tags.length > 0 ? tags.join(",") : undefined,
      },
    }),
} as const;
