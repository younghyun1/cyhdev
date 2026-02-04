// Health check response DTOs

/** Response from GET /api/healthcheck/state */
export interface HealthStateResponse {
  timestamp: string;
  server_uptime: string;
  responses_handled: number;
  users_logged_in: number;
  db_version: string;
  db_latency: string;
}

/** Response from GET /api/healthcheck/server */
export interface ServerHealthcheckResponse {
  build_time: string;
  axum_version: string;
  rust_version: string;
}

/** Response from GET /api/healthcheck/fastfetch - returns HTML string */
export type FastfetchResponse = string;
