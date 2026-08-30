// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  BatchListResponse,
  BatchStatusResponse,
  BatchUploadResponse,
  DeletePhotographsRequest,
  DeletePhotographsResponse,
  GetPhotographsResponse,
  Photograph,
  ReadPhotographResponse,
} from "../api-types";
import {
  appendQuery,
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createPhotographyMediaClient(transport: ApiTransport) {
  return {
    batchList: async (options: ApiRequestOptions = {}) => {
      const path = "/api/photographs/batches";
      const url = path;
      return requestJson<ApiResponse<BatchListResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    batchStatus: async (input: {
      readonly path: {
        readonly batch_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/batch/{batch_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<BatchStatusResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    batchUpload: async (input: {
      readonly body: FormData;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/photographs/batch-upload";
      const url = path;
      return requestJson<ApiResponse<BatchUploadResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
        body: input.body,
      });
    },
    deletePhotographs: async (input: {
      readonly body: DeletePhotographsRequest;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/photographs/delete";
      const url = path;
      return requestJson<ApiResponse<DeletePhotographsResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    getPhotographs: async (input: {
      readonly query?: {
        readonly page?: number;
        readonly page_size?: number;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/photographs/get";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<GetPhotographsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    readPhotograph: async (input: {
      readonly path: {
        readonly photograph_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/photographs/{photograph_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<ReadPhotographResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    uploadPhotograph: async (input: {
      readonly body: FormData;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/photographs/upload";
      const url = path;
      return requestJson<ApiResponse<Photograph>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
        body: input.body,
      });
    },
  } as const;
}
