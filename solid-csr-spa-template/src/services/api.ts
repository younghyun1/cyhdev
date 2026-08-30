import { setAuthenticated, setSuperuser, setUser } from "../state/auth";
import type { ServerHealthcheckResponse } from "../dtos/responses/health";
import { updateServerBuildInfo } from "../state/server_info";

export const API_URL = import.meta.env.VITE_API_URL || "";

const API_KEY = import.meta.env.VITE_API_KEY ?? "";

export function apiUrl(path: string) {
  return `${API_URL}${path}`;
}

declare const __BUILD_TIMESTAMP__: string;
declare const __APP_NAME__: string;

// Shape of /api/healthcheck/state (RootHandlerResponse wrapper)
export type HealthStateResponse = {
  success: boolean;
  data: {
    timestamp: string;
    server_uptime: string;
    responses_handled: number;
    users_logged_in: number;
    db_version: string;
    db_latency: string;
  };
  meta: {
    time_to_process: string;
    timestamp: string;
    metadata: unknown;
  };
};

const POST_LOGIN_REDIRECT_KEY = "post_login_redirect";

/** Read and clear the saved post-login redirect target. */
export function consumePostLoginRedirect(): string | null {
  try {
    const target = sessionStorage.getItem(POST_LOGIN_REDIRECT_KEY);
    sessionStorage.removeItem(POST_LOGIN_REDIRECT_KEY);
    return target && !target.startsWith("/login") ? target : null;
  } catch {
    return null;
  }
}

export async function apiFetch(path: string, options: RequestInit = {}) {
  options.headers = {
    ...options.headers,
    "x-api-key": API_KEY,
  };

  const response = await fetch(apiUrl(path), { credentials: "include", ...options });

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

  if (response.status === 401 || response.status === 403) {
    setAuthenticated(false);
    setUser(null);
    setSuperuser(false);

    if (!window.location.pathname.startsWith("/login")) {
      try {
        const currentUrl =
          window.location.pathname +
          window.location.search +
          window.location.hash;
        sessionStorage.setItem(POST_LOGIN_REDIRECT_KEY, currentUrl);
      } catch (err) {
        console.warn("Failed to persist post-login redirect target:", err);
      }
      window.location.href = `/login?next=${encodeURIComponent(
        window.location.pathname +
          window.location.search +
          window.location.hash,
      )}`;
    }

    throw new Error("Unauthorized – redirected to login");
  }

  return response;
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
