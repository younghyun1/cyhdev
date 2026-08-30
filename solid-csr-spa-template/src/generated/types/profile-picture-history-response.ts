// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ProfilePictureHistoryItem } from "./profile-picture-history-item";

export type ProfilePictureHistoryResponse = {
  readonly maximum_profile_pictures: number;
  readonly profile_pictures: ReadonlyArray<ProfilePictureHistoryItem>;
};
