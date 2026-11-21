// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type {
  ApiResponse,
  ArchiveAccessResponse,
  ArchiveDocumentResponse,
  ArchiveHistogramResponse,
  ArchiveReaderGrant,
  ArchiveReadersResponse,
  ArchiveSearchResponse,
  ArchiveStatusResponse,
  ArchiveTrendsResponse,
} from "../api-types";
import {
  appendQuery,
  interpolatePath,
  requestHeaders,
  requestJson,
  type ApiRequestOptions,
  type ApiTransport,
} from "../runtime";

export function createKcnaArchiveClient(transport: ApiTransport) {
  return {
    kcnaArchiveAccess: async (options: ApiRequestOptions = {}) => {
      const path = "/api/kcna-archive/access";
      const url = path;
      return requestJson<ApiResponse<ArchiveAccessResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveDocument: async (input: {
      readonly path: {
        readonly document_id: string;
      };
      readonly query?: {
        readonly q?: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/kcna-archive/documents/{document_id}", input.path);
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ArchiveDocumentResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveGrantReader: async (input: {
      readonly body: ArchiveReaderGrant;
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/admin/kcna-archive/readers";
      const url = path;
      return requestJson<ApiResponse<ArchiveReadersResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, true),
        signal: options.signal,
        body: JSON.stringify(input.body),
      });
    },
    kcnaArchiveHistogram: async (input: {
      readonly query?: {
        readonly bucket?: string;
        readonly category?: string;
        readonly from?: string;
        readonly lang?: string;
        readonly limit?: number;
        readonly offset?: number;
        readonly q?: string;
        readonly sort?: string;
        readonly source?: string;
        readonly to?: string;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/kcna-archive/histogram";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ArchiveHistogramResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveReaders: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/kcna-archive/readers";
      const url = path;
      return requestJson<ApiResponse<ArchiveReadersResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveReload: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/kcna-archive/reload";
      const url = path;
      return requestJson<ApiResponse<ArchiveStatusResponse>>(transport, url, {
        method: "POST",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveRevokeReader: async (input: {
      readonly path: {
        readonly user_id: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = interpolatePath("/api/admin/kcna-archive/readers/{user_id}", input.path);
      const url = path;
      return requestJson<ApiResponse<ArchiveReadersResponse>>(transport, url, {
        method: "DELETE",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveSearch: async (input: {
      readonly query?: {
        readonly bucket?: string;
        readonly category?: string;
        readonly from?: string;
        readonly lang?: string;
        readonly limit?: number;
        readonly offset?: number;
        readonly q?: string;
        readonly sort?: string;
        readonly source?: string;
        readonly to?: string;
      };
    } = {}, options: ApiRequestOptions = {}) => {
      const path = "/api/kcna-archive/search";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ArchiveSearchResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveStatus: async (options: ApiRequestOptions = {}) => {
      const path = "/api/admin/kcna-archive/status";
      const url = path;
      return requestJson<ApiResponse<ArchiveStatusResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
    kcnaArchiveTrends: async (input: {
      readonly query: {
        readonly bucket?: string;
        readonly category?: string;
        readonly from?: string;
        readonly lang?: string;
        readonly limit?: number;
        readonly offset?: number;
        readonly q?: string;
        readonly sort?: string;
        readonly source?: string;
        readonly term: ReadonlyArray<string>;
        readonly to?: string;
      };
    }, options: ApiRequestOptions = {}) => {
      const path = "/api/kcna-archive/trends";
      const url = appendQuery(path, input.query);
      return requestJson<ApiResponse<ArchiveTrendsResponse>>(transport, url, {
        method: "GET",
        headers: requestHeaders(options.headers, false),
        signal: options.signal,
      });
    },
  } as const;
}
