import { setAuthenticated, setSuperuser, setUser } from "../state/auth";
import type {
  ApiResponse,
  RootHandlerResponse,
  ServerHealthcheckResponse,
} from "../generated";
import { updateServerBuildInfo } from "../state/server_info";

export const API_URL = import.meta.env.VITE_API_URL || "";

export function apiUrl(path: string) {
  return `${API_URL}${path}`;
}

declare const __BUILD_TIMESTAMP__: string;
declare const __APP_NAME__: string;

export type HealthStateResponse = ApiResponse<RootHandlerResponse>;

const POST_LOGIN_REDIRECT_KEY = "post_login_redirect";

/** Read and clear the saved post-login redirect target. */
export function consumePostLoginRedirect(): string | null {
  try {
    const target = sessionStorage.getItem(POST_LOGIN_REDIRECT_KEY);
    sessionStorage.removeItem(POST_LOGIN_REDIRECT_KEY);
    return target?.startsWith("/")
      && !target.startsWith("//")
      && !target.startsWith("/login")
      ? target
      : null;
  } catch {
    return null;
  }
}

/** Saves a validated same-site route across an external authentication redirect. */
export function rememberPostLoginRedirect(target: string): void {
  if (!target.startsWith("/") || target.startsWith("//") || target.startsWith("/login")) {
    return;
  }
  try {
    sessionStorage.setItem(POST_LOGIN_REDIRECT_KEY, target);
  } catch {
    // Navigation can continue; the post-login fallback remains the home page.
  }
}

export async function apiFetch(path: string, options: RequestInit = {}) {
  const response = await fetch(apiUrl(path), {
    credentials: "include",
    ...options,
  });

  const builtTime = response.headers.get("x-server-built-time");
  const serverName = response.headers.get("x-server-name");
  const rustVersion = response.headers.get("x-server-rust-version");

  if (builtTime || serverName || rustVersion) {
    updateServerBuildInfo({
      built_time: builtTime ?? undefined,
      name: serverName ?? undefined,
      rust_version: rustVersion ?? undefined,
    });
  }

  if (response.status === 401) {
    handleUnauthorizedResponse();
    throw new Error("Unauthorized; redirected to login");
  }
  if (response.status === 403) {
    handleAdminForbiddenResponse(path);
  }

  return response;
}

/** Drops stale privileged UI immediately after a database-authoritative denial. */
export function handleAdminForbiddenResponse(path: string): void {
  if (path.startsWith("/api/admin/")) {
    setSuperuser(false);
  }
}

/** Clears local authority and redirects after an authenticated request fails. */
export function handleUnauthorizedResponse(): void {
  setAuthenticated(false);
  setUser(null);
  setSuperuser(false);

  if (window.location.pathname.startsWith("/login")) return;
  const currentUrl =
    window.location.pathname + window.location.search + window.location.hash;
  try {
    sessionStorage.setItem(POST_LOGIN_REDIRECT_KEY, currentUrl);
  } catch (error: unknown) {
    console.warn("Failed to persist post-login redirect target:", error);
  }
  window.location.href = `/login?next=${encodeURIComponent(currentUrl)}`;
}

// Helper for JSON GETs that also benefits from header-based build info
export async function apiGetJson<T = unknown>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const resp = await apiFetch(path, {
    method: options.method ?? "GET",
    credentials: options.credentials ?? "include",
    ...options,
  });
  if (!resp.ok) {
    throw new Error(`GET ${path} failed: ${resp.status}`);
  }
  return (await resp.json()) as T;
}

// Dedicated helper for /api/healthcheck/state
export async function fetchHealthState(): Promise<HealthStateResponse> {
  return apiGetJson<HealthStateResponse>("/api/healthcheck/state");
}

export async function fetchServerBuildInfo(): Promise<ServerHealthcheckResponse> {
  return apiGetJson<ServerHealthcheckResponse>("/api/healthcheck/server", {
    cache: "no-store",
    credentials: "omit",
  });
}
