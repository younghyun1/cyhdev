# OpenID Connect account linking

OpenID Connect is an optional login method for existing cyhdev accounts. Omitting every `OIDC_*` provider variable disables the provider, leaves startup independent of discovery, returns `enabled: false` from the status endpoint, and hides browser controls. It does not create local accounts and never links by email.

## Configuration

`PUBLIC_APP_ORIGIN` is the one canonical browser origin used by OIDC callbacks, verification links, password-reset links, CORS, and origin checks. Local defaults to `https://localhost:30737`; production defaults to `https://cyhdev.com`; development and staging require an explicit HTTPS value. It must be an exact origin without credentials, path, query, or fragment. Local mode alone permits an explicit loopback HTTP origin.

Enabling OIDC requires `OIDC_PROVIDER_NAME`, `OIDC_ISSUER_URL`, and `OIDC_CLIENT_ID`. `OIDC_CLIENT_SECRET` is optional and is read without trimming because it is opaque server credential material. The provider must register exactly `${PUBLIC_APP_ORIGIN}/api/auth/oidc/callback`, expose discovery and JWKS metadata, support Authorization Code flow, return an ID token, and grant the `openid` and `email` scopes with `email_verified=true`. Provider credentials and registrations are deployment work; this repository does not provision them.

## Login flow

The same-origin SPA posts to `/api/auth/oidc/login/start`. The IP limiter permits 10 starts per minute and 50 per hour before any pending state is allocated. The backend creates 256-bit state, nonce, and PKCE verifier material, sends only the S256 challenge to the provider, and retains the verifier in a fixed 512-entry RAM store for at most ten minutes. The callback consumes state once, exchanges the code through a no-redirect HTTP client, validates the JWT signature, issuer, audience, expiration, nonce, provider-verified email, and access-token hash when present, then resolves the account only by the stored issuer and subject pair. The local account must still be active and locally email-verified. A successful login rotates the process-local session and sets the host-only `Secure`, `HttpOnly`, `SameSite=Strict` cookie.

## Link and unlink flow

Link start requires a current locally verified session and records that user ID with pending state. `SameSite=Strict` intentionally withholds the session cookie on a cross-site provider callback, so the callback never mutates linkage. After provider validation it stores the claims behind a second 256-bit, five-minute, one-time RAM capability and redirects to `${PUBLIC_APP_ORIGIN}/edit-profile#oidc_link_token=...` with `Cache-Control: no-store` and `Referrer-Policy: no-referrer`. The SPA reads and removes the fragment before network work, then posts the capability from the same origin. The strict session is present on that post; the service requires the current user to match the user who started the flow and rechecks active and verified account state in the insert transaction.

Unlink requires the current local password, an active verified session, and a retained non-empty password hash. The transaction locks the account, rechecks the password snapshot, removes only the configured issuer link, and rotates the current session. Soft account deletion removes all OIDC identities before clearing other authority. The table uses UUIDv7 keys, full table-prefixed columns, byte-bounded issuer, subject, and email excerpts, named uniqueness on issuer-subject and account-issuer, indexed timestamps, and a fail-closed rollback when links exist.

## HTTP and TLS assumptions

`openidconnect` 4.0.1 is built without its HTTP-client features. A small adapter uses the workspace's reqwest 0.13 client and rustls path, rejects redirects, rejects plain HTTP outside local loopback, caps requests at 64 KiB and responses at 2 MiB, and applies a ten-second timeout. This avoids adding reqwest 0.12 or native TLS through the OIDC crate. The existing workspace already has rustls 0.23 and a legacy rustls 0.21 branch through AWS; OIDC adds no new TLS implementation. Discovery and JWKS are loaded at process startup when configured, so deployments must permit outbound HTTPS and provide the scratch image CA bundle.
