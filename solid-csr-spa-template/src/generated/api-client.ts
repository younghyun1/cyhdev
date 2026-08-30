// Generated from rust-be-template OpenAPI. Do not edit by hand.

import type { ApiTransport } from "./runtime";
import { createAccountClient } from "./clients/account";
import { createAuthorizationClient } from "./clients/authorization";
import { createBlogPostsClient } from "./clients/blog-posts";
import { createBlogSocialClient } from "./clients/blog-social";
import { createI18nClient } from "./clients/i18n";
import { createLiveChatClient } from "./clients/live-chat";
import { createOidcClient } from "./clients/oidc";
import { createPhotographyMediaClient } from "./clients/photography-media";
import { createPhotographySocialClient } from "./clients/photography-social";
import { createReferenceClient } from "./clients/reference";
import { createWasmClient } from "./clients/wasm";

export { ApiContractError } from "./runtime";
export type { ApiRequestOptions, ApiTransport } from "./runtime";

export function createApiClient(transport: ApiTransport) {
  return {
    ...createAccountClient(transport),
    ...createAuthorizationClient(transport),
    ...createBlogPostsClient(transport),
    ...createBlogSocialClient(transport),
    ...createI18nClient(transport),
    ...createLiveChatClient(transport),
    ...createOidcClient(transport),
    ...createPhotographyMediaClient(transport),
    ...createPhotographySocialClient(transport),
    ...createReferenceClient(transport),
    ...createWasmClient(transport),
  } as const;
}

export type ApiClient = ReturnType<typeof createApiClient>;
