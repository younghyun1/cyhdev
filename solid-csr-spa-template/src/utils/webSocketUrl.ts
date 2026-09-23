/**
 * Absolute `ws:`/`wss:` URL for a backend WebSocket path.
 *
 * `apiBase` follows `VITE_API_URL`: empty means same-origin, and a configured
 * base keeps its path prefix, matching how `apiUrl` joins HTTP paths. The
 * socket scheme mirrors the resolved HTTP scheme, so an HTTPS page never opens
 * a plaintext socket. Browsers resolve relative WebSocket URLs only in recent
 * versions, so the result is always absolute.
 */
export function webSocketUrl(
  path: string,
  apiBase: string,
  pageOrigin: string,
): string {
  const url = new URL(`${apiBase.replace(/\/+$/, "")}${path}`, pageOrigin);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}
