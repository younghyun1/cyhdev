// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { UserInfo } from "./user-info";
import type { UserProfilePicture } from "./user-profile-picture";

export type MeResponse = {
  readonly axum_version: string;
  readonly build_time: string;
  readonly rust_version: string;
  readonly user_info: UserInfo | null;
  readonly user_profile_picture: UserProfilePicture | null;
};
