export { authApi, userApi } from "./contracts/account";
export { oidcApi } from "./contracts/oidc";
export { blogApi } from "./contracts/blog";
export { adminApi, i18nApi, wasmModuleApi } from "./contracts/misc";
export { authorizationAdminApi } from "./contracts/authorization";
export { forumApi } from "./contracts/forum";
export { photographyApi } from "./contracts/photography";
export {
  dropdownApi,
  geoApi,
  geoIpApi,
  healthApi,
  visitorBoardApi,
} from "./contracts/reference";

export type { IpInfo, WasmModuleItem } from "../generated";
