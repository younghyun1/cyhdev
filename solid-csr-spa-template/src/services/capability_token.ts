/**
 * One-time capabilities in email links and OIDC link completion are 256 random bits
 * encoded as 43 unpadded base64url characters. The server stores only their SHA-256.
 */
const CAPABILITY_TOKEN = /^[A-Za-z0-9_-]{43}$/;

/** Whether a fragment value has the canonical one-time capability shape. */
export function isCapabilityToken(value: string): boolean {
  return CAPABILITY_TOKEN.test(value);
}
