import type { ResolveMediaCleanupRequest } from "../../generated";
import { contractApi } from "../account_api";

/** Only the configured application bucket can be admitted to cleanup maintenance. */
export const MEDIA_CLEANUP_BUCKET = "cyhdev-img" as const;

export type RetentionStatusQuery = {
  readonly after_next_attempt_at?: string;
  readonly after_notification_id?: string;
  readonly limit?: number;
};

/** Credentialed, generated-contract operations used by the administration page. */
export const adminOperationsApi = {
  retentionStatus: (
    query: RetentionStatusQuery = {},
    signal?: AbortSignal,
  ) =>
    contractApi.retentionNotificationStatus(
      { query },
      signal === undefined ? {} : { signal },
    ),
  retryRetentionNotification: (notificationId: string) =>
    contractApi.retryRetentionNotification({
      path: { notification_id: notificationId },
    }),
  unresolvedMediaCleanup: (signal?: AbortSignal) =>
    contractApi.unresolvedMediaCleanup(
      signal === undefined ? {} : { signal },
    ),
  resolveMediaCleanup: (
    cleanupId: string,
    body: ResolveMediaCleanupRequest,
  ) =>
    contractApi.resolveMediaCleanup({
      path: { cleanup_id: cleanupId },
      body,
    }),
  hardPurgeAccount: (userId: string) =>
    contractApi.hardPurgeAccount({ path: { user_id: userId } }),
  syncI18n: () => contractApi.syncI18nCache(),
} as const;

export type AdminOperationsApi = typeof adminOperationsApi;
