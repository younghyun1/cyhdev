// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ArchiveServiceState } from "./archive-service-state";
import type { ArchiveSnapshotStatus } from "./archive-snapshot-status";

export type ArchiveStatusResponse = {
  readonly lastReloadFailed: boolean;
  readonly reloading: boolean;
  readonly snapshot?: ArchiveSnapshotStatus | null;
  readonly state: ArchiveServiceState;
};
