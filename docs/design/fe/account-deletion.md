# Account deletion

The Edit Profile danger panel is the only self-service account-deletion entry point. It calls `DELETE /api/auth/account` through the generated credentialed client, so the browser sends the existing session cookie and a JSON body containing the current password. The password is never persisted in browser storage; the input signal is cleared after every attempt.

Deletion requires two independent inputs: the current password and a checked acknowledgement that deletion is irreversible and retained authored content will use anonymous attribution. The submit control remains disabled until both are present and while the request is pending. HTTP 422 password-confirmation failures remain in the panel and do not trigger the unauthorized-session redirect; HTTP 401 remains reserved for an invalid session.

On success, the panel renders the server-returned `purge_after` timestamp using the browser locale. The receipt remains visible for five seconds, after which the browser clears account, role, and authentication state and returns to the public home page. A user can leave immediately with the same state-clearing action.

The generated account client also contains `POST /api/admin/users/{user_id}/hard-purge` and its explicit object-cleanup result types. No admin control invokes it yet; administrative lifecycle controls belong to the separately authorized account-administration interface.
