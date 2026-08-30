# Browser authentication

Vite environment variables are part of the public browser bundle. They must contain only public configuration such as the API origin; credentials, signing material, provider client secrets, and static API keys must remain on the backend.

The browser authenticates to this application with the backend-issued session cookie. HTTP requests use `credentials: "include"`, multipart XMLHttpRequests set `withCredentials`, and WebSocket authentication relies on cookies sent during the opening handshake. Browser code must not add an API key header, put credentials in a WebSocket message, or persist session tokens in Web Storage.

The backend owns the security boundary. It must issue session cookies with `HttpOnly`, `Secure`, and an appropriate `SameSite` policy; restrict credentialed CORS to explicit origins; validate WebSocket `Origin` headers; and protect unsafe cookie-authenticated methods against cross-site request forgery. A third-party API that requires a secret must be called by the backend through a narrow application endpoint, not directly by the browser.
