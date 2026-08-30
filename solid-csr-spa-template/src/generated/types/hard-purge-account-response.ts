// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ProfileObjectCleanupFailure } from "./profile-object-cleanup-failure";

export type HardPurgeAccountResponse = {
  readonly hard_purged_at: string;
  readonly profile_cleanup_failures: ReadonlyArray<ProfileObjectCleanupFailure>;
  readonly profile_cleanup_remaining: number;
  readonly profile_metadata_deleted: number;
  readonly profile_objects_deleted: number;
  readonly user_id: string;
};
