# Account profile editing

Edit Profile sends the complete editable identity set to `PATCH /api/auth/profile`: display name, country, language, optional subdivision, and the current password. Email is rendered read-only and is absent from the request type. Country and language choices reuse the existing generated reference-data clients; changing country reloads subdivisions and clears a selection that does not belong to the new country.

The form applies the backend username rules before submission and renders errors beside the responsible field. Wrong-password and duplicate-name responses remain local to the password and display-name fields. A successful response updates the shared authentication state without another account request; the password signal is cleared after every attempt.

Profile-picture uploads add a new active entry instead of discarding all history. The panel reloads both the bounded history and current-account view after upload, selection, or deletion. At most eight entries are rendered, the active entry is marked, and deleting an entry requires confirmation. A deletion receipt reports deferred object cleanup so the UI can distinguish a completed metadata deletion from remaining administrative cleanup.

The generated account client includes bounded unresolved-cleanup listing and optimistic cleanup resolution contracts for the future account-administration interface. The self-service profile page does not expose those superuser operations.
