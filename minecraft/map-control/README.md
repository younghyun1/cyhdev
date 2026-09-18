# Cyhdev map control

This Paper plugin exposes squaremap's persistent player visibility through a Unix socket. The website and Minecraft must run as the same OS user. It depends on squaremap and uses its public `PlayerManager` API. It does not dispatch console commands, listen on TCP, or maintain a second visibility database. It targets Paper 26.3 and Java 25; Folia is not supported.

The socket is `plugins/CyhdevMapControl/control.sock` relative to Minecraft's working directory. Its parent directory is mode `0700`; the socket is `0600`. The plugin processes one connection at a time with a three-second deadline, a 64-byte request limit, and at most 1000 status rows. Bukkit and squaremap calls run on the main server thread. Expired queued requests are cancelled; as with any external mutation, a lost acknowledgement can still follow a completed change. Do not retry automatically.

## Build and install

Compile against the installed Paper API and squaremap jar, without bundling either dependency. The following commands run in Bash from this directory; set `MC_SERVER` to the Minecraft directory. The first classpath entry pins the actual Paper API instead of accidentally compiling against older cached server libraries.

```bash
MC_SERVER="$HOME/mcserver"
MAP_CLASSPATH="$MC_SERVER/libraries/io/papermc/paper/paper-api/26.3.build.16-alpha/paper-api-26.3.build.16-alpha.jar"
MAP_CLASSPATH="$MAP_CLASSPATH:$MC_SERVER/plugins/squaremap-paper-mc26.3-1.4.1-SNAPSHOT+ac71dd2.jar"
while IFS= read -r dependency; do MAP_CLASSPATH="$MAP_CLASSPATH:$dependency"; done < <(rg --files "$MC_SERVER/libraries" -g '*.jar')
mkdir -p target/classes
javac -Xlint:all,-classfile -Werror -cp "$MAP_CLASSPATH" -d target/classes src/com/cyhdev/minecraft/MapControl.java
jar --create --file target/cyhdev-map-control.jar -C target/classes . -C . plugin.yml
cp target/cyhdev-map-control.jar "$MC_SERVER/plugins/"
```

`-classfile` suppresses missing optional annotation warnings in external dependencies; source warnings remain errors. Set `MINECRAFT_MAP_CONTROL_SOCKET` in the website environment to the absolute socket path. Restart Minecraft once to load the plugin, then deploy the website normally. Subsequent toggles need no restart. Removing the variable disables website visibility controls; removing the plugin jar takes effect on the next restart. Existing squaremap visibility preferences remain intact.

## Protocol and verification

Each connection accepts one ASCII line: `STATUS`, `HIDE <canonical UUID>`, or `SHOW <canonical UUID>`, terminated by a newline. A successful mutation returns `OK\n`. Status returns `OK\n` followed by zero or more `<UUID> <0|1>\n` rows; `1` means squaremap's explicit hidden state. The server closes the connection after the response. Invalid requests or offline targets return `ERROR\n`; deadlines may instead close the connection without a response. Visibility allowed does not override spectator, invisibility, equipment, or world tracker settings.

The ignored Rust tests `inspect_live_map_control` and `inspect_empty_plugin_rejections` exercise the real plugin. The latter requires an isolated Paper server with no players; it checks unsupported commands, invalid and offline targets, oversized requests, incomplete-frame timeout, and recovery. An SSH Unix-socket forward can connect the local Rust tests to a remote development instance. Normal tests cover strict acknowledgements, response bounds, and status parsing without a running JVM.
