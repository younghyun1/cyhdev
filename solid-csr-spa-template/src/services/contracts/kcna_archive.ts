import {
  type ArchiveAccessResponse,
  type ArchiveBucket,
  type ArchiveDocumentResponse,
  type ArchiveHistogramResponse,
  type ArchiveReadersResponse,
  type ArchiveSearchResponse,
  type ArchiveStatusResponse,
  type ArchiveTrendsResponse,
} from "../../generated";
import { PAGE_SIZE } from "../../utils/kcna_archive/constants";
import { queryParams, queryString, type ArchiveQuery } from "../../utils/kcna_archive/query";
import { contractApi } from "../account_api";
import { apiUrl } from "../api";

export type {
  ArchiveAccessResponse,
  ArchiveBucket,
  ArchiveCopyLink,
  ArchiveDocumentResponse,
  ArchiveHistogramBucket,
  ArchiveHistogramResponse,
  ArchiveLanguage,
  ArchiveLanguageFacet,
  ArchiveFacets,
  ArchivePrecision,
  ArchiveReaderItem,
  ArchiveReadersResponse,
  ArchiveSearchItem,
  ArchiveSearchResponse,
  ArchiveServiceState,
  ArchiveSnapshotStatus,
  ArchiveSource,
  ArchiveSourceFacet,
  ArchiveStatusResponse,
  ArchiveTextSegment,
  ArchiveTranslationLink,
  ArchiveTrendSeries,
  ArchiveTrendsResponse,
} from "../../generated";

const ARCHIVE_ROOT = "/api/kcna-archive";

export type ExportFormat = "csv" | "jsonl";

/** Per-request options; the signal lets a caller drop a request it no longer needs. */
export interface ArchiveRequestOptions {
  readonly signal?: AbortSignal;
}

const requestOptions = (options: ArchiveRequestOptions) =>
  options.signal === undefined ? {} : { signal: options.signal };

/**
 * A same-origin link the browser follows to download the current result set.
 * The server answers with a file attachment, not the JSON envelope, so this is
 * a link and not a client call.
 */
export const exportHref = (query: ArchiveQuery, format: ExportFormat): string =>
  apiUrl(`${ARCHIVE_ROOT}/export?${queryString(query, { format })}`);

export const kcnaArchiveApi = {
  async search(
    query: ArchiveQuery,
    offset: number,
    options: ArchiveRequestOptions = {},
  ): Promise<ArchiveSearchResponse> {
    const response = await contractApi.kcnaArchiveSearch(
      {
        query: {
          ...queryParams(query),
          offset: offset === 0 ? undefined : offset,
          limit: PAGE_SIZE,
        },
      },
      requestOptions(options),
    );
    return response.data;
  },

  async histogram(
    query: ArchiveQuery,
    bucket: ArchiveBucket,
    options: ArchiveRequestOptions = {},
  ): Promise<ArchiveHistogramResponse> {
    const response = await contractApi.kcnaArchiveHistogram(
      { query: { ...queryParams(query), bucket } },
      requestOptions(options),
    );
    return response.data;
  },

  /** One document; `text` marks its matches in the title and body. */
  async document(
    id: string,
    text: string,
    options: ArchiveRequestOptions = {},
  ): Promise<ArchiveDocumentResponse> {
    const response = await contractApi.kcnaArchiveDocument(
      { path: { document_id: id }, query: text === "" ? undefined : { q: text } },
      requestOptions(options),
    );
    return response.data;
  },

  /** Term frequencies per bucket for up to four terms; the `term` parameter repeats. */
  async trends(
    terms: ReadonlyArray<string>,
    query: ArchiveQuery,
    bucket: ArchiveBucket,
    options: ArchiveRequestOptions = {},
  ): Promise<ArchiveTrendsResponse> {
    // The search text does not apply to trends: the terms are the text.
    const response = await contractApi.kcnaArchiveTrends(
      { query: { ...queryParams({ ...query, text: "" }), bucket, term: terms } },
      requestOptions(options),
    );
    return response.data;
  },

  /** Whether the signed-in visitor may read the archive. */
  async access(options: ArchiveRequestOptions = {}): Promise<ArchiveAccessResponse> {
    const response = await contractApi.kcnaArchiveAccess(requestOptions(options));
    return response.data;
  },

  /** The accounts allowed to read the archive; superusers only. */
  async readers(options: ArchiveRequestOptions = {}): Promise<ArchiveReadersResponse> {
    const response = await contractApi.kcnaArchiveReaders(requestOptions(options));
    return response.data;
  },

  /** Allows an account to read; resolves with the list as it now stands. */
  async grantReader(
    userId: string,
    options: ArchiveRequestOptions = {},
  ): Promise<ArchiveReadersResponse> {
    const response = await contractApi.kcnaArchiveGrantReader(
      { body: { userId } },
      requestOptions(options),
    );
    return response.data;
  },

  /** Removes an account from the list; resolves with the list as it now stands. */
  async revokeReader(
    userId: string,
    options: ArchiveRequestOptions = {},
  ): Promise<ArchiveReadersResponse> {
    const response = await contractApi.kcnaArchiveRevokeReader(
      { path: { user_id: userId } },
      requestOptions(options),
    );
    return response.data;
  },

  async status(options: ArchiveRequestOptions = {}): Promise<ArchiveStatusResponse> {
    const response = await contractApi.kcnaArchiveStatus(requestOptions(options));
    return response.data;
  },

  /** Reads the snapshot from disk again; resolves with the status after the swap. */
  async reload(options: ArchiveRequestOptions = {}): Promise<ArchiveStatusResponse> {
    const response = await contractApi.kcnaArchiveReload(requestOptions(options));
    return response.data;
  },
} as const;
