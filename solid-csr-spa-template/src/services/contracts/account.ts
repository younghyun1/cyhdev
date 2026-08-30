import type {
  CheckIfUserExistsRequest,
  LoginRequest,
  ResetPasswordProcessRequest,
  ResetPasswordRequest,
  SignupRequest,
} from "../../generated";
import { contractApi } from "../account_api";

export const authApi = {
  signup: (body: SignupRequest) => contractApi.signup({ body }),
  checkIfUserExists: (body: CheckIfUserExistsRequest) =>
    contractApi.checkIfUserExists({ body }),
  login: (body: LoginRequest) => contractApi.login({ body }),
  resetPasswordRequest: (body: ResetPasswordRequest) =>
    contractApi.resetPasswordRequest({ body }),
  resetPassword: (body: ResetPasswordProcessRequest) =>
    contractApi.resetPassword({ body }),
  verifyUserEmail: (emailValidationTokenId: string) =>
    contractApi.verifyUserEmail({
      query: { email_validation_token_id: emailValidationTokenId },
    }),
  me: () => contractApi.me(),
  isSuperuser: () => contractApi.isSuperuser(),
  logout: () => contractApi.logout(),
  uploadProfilePicture: (body: FormData) =>
    contractApi.uploadProfilePicture({ body }),
} as const;

export const userApi = {
  getPublicUserInfo: (userName: string) =>
    contractApi.getUserInfo({ path: { user_name: userName } }),
  uploadProfilePicture: (body: FormData) =>
    contractApi.uploadProfilePicture({ body }),
} as const;
