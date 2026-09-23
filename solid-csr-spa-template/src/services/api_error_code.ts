import { ApiContractError } from "../generated";

/** Backend error code for a correct password on an account whose email is unverified. */
export const LOGIN_EMAIL_NOT_VERIFIED = 68;
/** Backend error code for a current-password confirmation that did not match. */
export const PASSWORD_CONFIRMATION_FAILED = 55;

/** Reads the stable numeric `error_code` from a failed contract call, if it has one. */
export function apiErrorCode(error: unknown): number | null {
  if (!(error instanceof ApiContractError) || !error.body) return null;
  try {
    const parsed: unknown = JSON.parse(error.body);
    if (typeof parsed === "object" && parsed !== null && "error_code" in parsed) {
      const code = (parsed as { readonly error_code?: unknown }).error_code;
      return typeof code === "number" ? code : null;
    }
  } catch {
    return null;
  }
  return null;
}
