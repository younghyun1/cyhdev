import { ApiContractError } from "../../generated";
import type { UiTextKey } from "../../i18n/keys";
import { apiErrorCode } from "../../services/api_error_code";

/** Stable backend error codes of the archive endpoints. */
const CODE = {
  documentNotFound: 112,
  notInstalled: 113,
  loading: 114,
  loadFailed: 115,
  reloadBusy: 116,
  reloadFailed: 117,
} as const;

/** Why a request failed, independent of the display language. */
export type FailureKind =
  | "notFound"
  | "badRequest"
  | "notInstalled"
  | "loading"
  | "loadFailed"
  | "busy"
  | "reloadFailed"
  | "unavailable"
  | "network";

export const failureKind = (error: unknown): FailureKind => {
  if (error instanceof ApiContractError) {
    switch (apiErrorCode(error)) {
      case CODE.documentNotFound:
        return "notFound";
      case CODE.notInstalled:
        return "notInstalled";
      case CODE.loading:
        return "loading";
      case CODE.loadFailed:
        return "loadFailed";
      case CODE.reloadBusy:
        return "busy";
      case CODE.reloadFailed:
        return "reloadFailed";
      default:
        break;
    }
    if (error.status === 404) return "notFound";
    if (error.status === 400) return "badRequest";
    if (error.status === 409) return "busy";
    return "unavailable";
  }
  return "network";
};

/**
 * The archive has no snapshot to answer from. This is a state of the server,
 * not a fault of the request, so pages show it as a plain notice.
 */
export const isNotReady = (kind: FailureKind): boolean =>
  kind === "notInstalled" || kind === "loading" || kind === "loadFailed";

export const failureKey = (kind: FailureKind): UiTextKey => {
  switch (kind) {
    case "notFound":
      return "kcna.failure.not_found";
    case "badRequest":
      return "kcna.failure.bad_request";
    case "notInstalled":
      return "kcna.state.not_installed";
    case "loading":
      return "kcna.state.loading_snapshot";
    case "loadFailed":
      return "kcna.state.failed";
    case "busy":
      return "kcna.status.reload_busy";
    case "reloadFailed":
      return "kcna.status.reload_failed";
    case "unavailable":
      return "kcna.failure.unavailable";
    case "network":
      return "kcna.failure.network";
  }
};
