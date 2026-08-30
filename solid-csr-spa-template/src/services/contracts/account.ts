import type {
  DeleteAccountRequest,
  LoginRequest,
  ResetPasswordProcessRequest,
  ResetPasswordRequest,
  ResolveMediaCleanupRequest,
  SignupRequest,
  UpdateProfileRequest,
  VerifyUserEmailRequest,
} from "../../generated";
import { contractApi } from "../account_api";

export const authApi = {
  signup: (body: SignupRequest) => contractApi.signup({ body }),
  login: (body: LoginRequest) => contractApi.login({ body }),
  resetPasswordRequest: (body: ResetPasswordRequest) =>
    contractApi.resetPasswordRequest({ body }),
  resetPassword: (body: ResetPasswordProcessRequest) =>
    contractApi.resetPassword({ body }),
  verifyUserEmail: (body: VerifyUserEmailRequest) =>
    contractApi.verifyUserEmail({ body }),
  me: () => contractApi.me(),
  isSuperuser: () => contractApi.isSuperuser(),
  logout: () => contractApi.logout(),
  deleteAccount: (body: DeleteAccountRequest) =>
    contractApi.deleteAccount({ body }),
  updateProfile: (body: UpdateProfileRequest) =>
    contractApi.updateProfile({ body }),
  uploadProfilePicture: (body: FormData) =>
    contractApi.uploadProfilePicture({ body }),
} as const;

export const adminAccountApi = {
  hardPurgeAccount: (userId: string) =>
    contractApi.hardPurgeAccount({ path: { user_id: userId } }),
  unresolvedMediaCleanup: () => contractApi.unresolvedMediaCleanup(),
  resolveMediaCleanup: (
    cleanupId: string,
    body: ResolveMediaCleanupRequest,
  ) =>
    contractApi.resolveMediaCleanup({
      path: { cleanup_id: cleanupId },
      body,
    }),
} as const;

export const userApi = {
  getPublicUserInfo: (userName: string) =>
    contractApi.getUserInfo({ path: { user_name: userName } }),
  uploadProfilePicture: (body: FormData) =>
    contractApi.uploadProfilePicture({ body }),
  profilePictures: () => contractApi.listProfilePictures(),
  selectProfilePicture: (profilePictureId: string) =>
    contractApi.selectProfilePicture({
      path: { profile_picture_id: profilePictureId },
    }),
  deleteProfilePicture: (profilePictureId: string) =>
    contractApi.deleteProfilePicture({
      path: { profile_picture_id: profilePictureId },
    }),
} as const;
