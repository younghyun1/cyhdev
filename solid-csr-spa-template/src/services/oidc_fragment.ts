export type OidcFragmentResult =
  | { readonly kind: "none" }
  | { readonly kind: "login-success" }
  | { readonly kind: "link-ready"; readonly completionToken: string }
  | { readonly kind: "failed" };

const COMPLETION_TOKEN = /^[A-Za-z0-9_-]{43}$/;

/** Parses only the OIDC fragment namespace; unrelated page anchors are untouched. */
export function parseOidcFragment(fragment: string): OidcFragmentResult {
  const raw = fragment.startsWith("#") ? fragment.slice(1) : fragment;
  if (!raw) return { kind: "none" };
  const params = new URLSearchParams(raw);
  const completionToken = params.get("oidc_link_token");
  if (completionToken !== null) {
    return COMPLETION_TOKEN.test(completionToken)
      ? { kind: "link-ready", completionToken }
      : { kind: "failed" };
  }
  const outcome = params.get("oidc");
  if (outcome === "success") return { kind: "login-success" };
  if (outcome === "failed") return { kind: "failed" };
  return { kind: "none" };
}

/** Reads an OIDC fragment once and removes it before any network or rendering work. */
export function consumeOidcFragment(): OidcFragmentResult {
  if (typeof window === "undefined") return { kind: "none" };
  const result = parseOidcFragment(window.location.hash);
  if (result.kind !== "none") {
    window.history.replaceState(
      null,
      "",
      `${window.location.pathname}${window.location.search}`,
    );
  }
  return result;
}
