const CANONICAL_UUID =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export type EmailVerificationLinkState =
  | { readonly kind: "ready"; readonly token: string }
  | { readonly kind: "missing" }
  | { readonly kind: "invalid" };

/**
 * Reads the fragment exactly once and removes it before the component can
 * initiate network work. The returned state is the only retained token copy.
 */
export function consumeEmailVerificationFragment(
  location: Location,
  history: History,
): EmailVerificationLinkState {
  const fragment = location.hash.startsWith("#")
    ? location.hash.slice(1)
    : location.hash;
  const entries = [...new URLSearchParams(fragment).entries()];
  const onlyEntry = entries.length === 1 ? entries[0] : undefined;
  const token = onlyEntry?.[0] === "token" ? onlyEntry[1] : undefined;
  const state: EmailVerificationLinkState =
    token === undefined
      ? fragment.length === 0
        ? { kind: "missing" }
        : { kind: "invalid" }
      : CANONICAL_UUID.test(token)
        ? { kind: "ready", token }
        : { kind: "invalid" };

  // This route accepts no query state; scrub both query and fragment so a
  // malformed or legacy token cannot remain in browser history.
  history.replaceState(history.state, "", location.pathname);
  return state;
}
