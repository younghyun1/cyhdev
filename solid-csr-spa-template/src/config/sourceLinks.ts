const DEFAULT_REPOSITORY_SOURCE_BASE_URL =
  "https://github.com/younghyun1/cyhdev/blob/main";

type RepositoryPackage =
  | "rust-be-template"
  | "solid-csr-spa-template"
  | "tools";

export type RepositorySourcePath =
  | `${RepositoryPackage}/${string}`
  | "Cargo.toml"
  | "rust-toolchain.toml";

function normalizeRepositorySourceBaseUrl(configuredUrl: string | undefined): string {
  const candidate = configuredUrl?.trim() || DEFAULT_REPOSITORY_SOURCE_BASE_URL;

  try {
    const parsedUrl = new URL(candidate);
    if (parsedUrl.protocol !== "https:" && parsedUrl.protocol !== "http:") {
      return DEFAULT_REPOSITORY_SOURCE_BASE_URL;
    }

    parsedUrl.hash = "";
    parsedUrl.search = "";
    return parsedUrl.toString().replace(/\/+$/, "");
  } catch {
    return DEFAULT_REPOSITORY_SOURCE_BASE_URL;
  }
}

export const repositorySourceBaseUrl = normalizeRepositorySourceBaseUrl(
  import.meta.env.VITE_REPOSITORY_SOURCE_BASE_URL,
);

export const repositorySourcePaths = {
  accountApi: "rust-be-template/src/features/accounts/api/mod.rs",
  accountService: "rust-be-template/src/features/accounts/service/mod.rs",
  accountRepository: "rust-be-template/src/features/accounts/repository/mod.rs",
  accountDomain: "rust-be-template/src/features/accounts/domain/mod.rs",
  frontendAuthState: "solid-csr-spa-template/src/state/auth.ts",
  frontendLogin: "solid-csr-spa-template/src/pages/login.tsx",
  generatedApiClient: "solid-csr-spa-template/src/generated/api-client.ts",
  generatedApiTypes: "solid-csr-spa-template/src/generated/api-types.ts",
  serverRouter: "rust-be-template/src/routers/main_router.rs",
  staticAssetServer:
    "rust-be-template/src/routers/main_router/static_assets.rs",
  databaseSchema: "rust-be-template/src/schema.rs",
  blogBackend: "rust-be-template/src/handlers/blog/mod.rs",
  blogFrontend: "solid-csr-spa-template/src/pages/posts/List.tsx",
  photographyBackend: "rust-be-template/src/handlers/photography/mod.rs",
  photographyFrontend: "solid-csr-spa-template/src/pages/photographs.tsx",
  liveChatBackend: "rust-be-template/src/domain/live_chat/mod.rs",
  liveChatFrontend: "solid-csr-spa-template/src/services/live_chat.ts",
  backendI18n: "rust-be-template/src/domain/i18n/mod.rs",
  frontendI18n: "solid-csr-spa-template/src/i18n/keys.ts",
  containerBuild: "rust-be-template/Dockerfile",
  rootCommands: "tools/xtask/src/main.rs",
  workspaceManifest: "Cargo.toml",
  rustToolchain: "rust-toolchain.toml",
} as const satisfies Record<string, RepositorySourcePath>;

/** Builds a provider URL from a stable path relative to the monorepo root. */
export function repositorySourceUrl(path: RepositorySourcePath): string {
  const encodedPath = path
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/");
  return `${repositorySourceBaseUrl}/${encodedPath}`;
}
