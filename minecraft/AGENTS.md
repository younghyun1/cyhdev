# Minecraft integration

Read [map-control README](map-control/README.md) before plugin changes and [backend controls](../docs/architecture/be/minecraft-controls.md) before changes spanning the website and server. The Java Paper plugin owns squaremap visibility through squaremap's API; Rust controls and frontend admin screens are separate consumers.

Preserve the Unix-socket protocol, same-OS-user requirement, directory/socket permissions, strict UUID parsing, request/response bounds, and deadlines. Bukkit/squaremap calls must execute on the server thread. A lost acknowledgement may follow a successful mutation; do not add automatic mutation retries. Do not expose TCP or substitute console-command dispatch.

The plugin targets Paper 26.3 and Java 25; Folia is unsupported. Verify Java tools before use. The README's build example ends with copying the jar into a live server: for local verification, stop after compilation and packaging. Do not copy, restart, or invoke live control sockets without deployment/test scope. Compile against the actual Paper and squaremap dependencies; do not bundle them.

Normal Rust protocol tests do not need a JVM. Ignored live tests require a configured server, and rejection tests require an isolated server without players. Record unavailable Java/server prerequisites rather than claiming root Clippy verifies this plugin.
