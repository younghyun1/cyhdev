import type {
  OidcLinkCompleteRequest,
  OidcUnlinkRequest,
} from "../../generated";
import { contractApi } from "../account_api";

export const oidcApi = {
  status: () => contractApi.oidcStatus(),
  startLogin: () => contractApi.startOidcLogin(),
  startLink: () => contractApi.startOidcLink(),
  completeLink: (body: OidcLinkCompleteRequest) =>
    contractApi.completeOidcLink({ body }),
  unlink: (body: OidcUnlinkRequest) => contractApi.unlinkOidc({ body }),
} as const;
