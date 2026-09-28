# Minecraft terrain explorer

The public `/minecraft` explorer reuses squaremap's rendered tiles and published player snapshots. Its own Leaflet client never executes the generated map's JavaScript in the authenticated website origin. The original squaremap application remains available under its existing opaque-origin sandbox. Public map observations and waypoint reads are independent of administrator server controls.

## World observations

`POST /api/minecraft/map/query` accepts a closed `catalog`, `area`, or `blocks` request. `MINECRAFT_WORLD_SOCKET` selects the same-user Unix socket exposed by the Paper plugin; an absent setting disables observations while leaving squaremap browsing available. Requests contain dimension keys and coordinates, never filesystem paths or commands. The Rust service bounds request and response frames to 4096 bytes and 2 MiB, uses a single nonwaiting operation slot, imposes a one-second cooldown, and applies a twenty-second deadline. A cancelled operation retains its reservation until twenty-one seconds after it began so another request cannot overlap game-side work. The plugin has a fifteen-second deadline and a socket with mode `0600` inside the existing mode `0700` directory.

The catalog contains at most sixteen squaremap-enabled worlds and 4096 block identifiers. Area queries cover at most eight by eight chunks and return up to 1024 four-block samples and 256 generated structures. Each sample contains surface elevation and a biome; an optional Y value selects the biome slice without changing the elevation meaning. Block queries cover at most four by four chunks, an inclusive vertical range of at most 512 blocks, and at most 512 exact matches. Empty or missing chunks are counted rather than generated. Incomplete results carry a truncation flag and every response carries its sampling timestamp.

Chunk loading uses Paper's asynchronous non-generating API, with one load outstanding and at most one snapshot scheduled per tick. Snapshot analysis occurs on the dedicated worker. No full-world index, continuous pan-triggered scan, or unbounded cache is introduced. Results are area observations rather than claims about the entire dimension.

## Saved structures

Paper's ordinary structure lookup follows reference chunks and can synchronously load them. The explorer instead reads each queried chunk's saved `structures.starts` metadata from the dimension-specific region directory. It combines the bounding boxes of saved structure pieces and never follows references into other chunks. A structure appears when its start chunk falls inside the scanned area; structure metadata reflects the last world save and may remain after players dismantle the structure.

The selective NBT reader caps compressed data at 1 MiB, decompressed data at 8 MiB, nesting at 64, and tag processing at 100,000. It validates chunk coordinates, bounding boxes, region allocations, file type, and header stability. It supports gzip, zlib, and uncompressed region chunks. Missing saves, external chunk storage, unsupported compression, malformed data, or detected concurrent header changes produce explicitly incomplete structure coverage rather than a false complete empty result.

## Waypoints

Waypoint reads are public and bounded to 256 records per dimension. Names and descriptions are plain text, bounded to eighty and five hundred characters; coordinates are restricted to supported world bounds. Creation, replacement, and deletion require the established session and trusted-origin guards and a database-current administrator authority lease held through the write.

The PostgreSQL repository serializes slot allocation using a dimension lock row. A unique dimension-and-slot constraint combined with slots restricted to zero through 255 enforces the capacity under concurrent writes. UUID identifiers use the database's UUIDv7 default. Waypoints are public shared map records, not copies of player identities or game-server data. No seed or management credential is returned to the browser.

## Activation and verification

Compile and install the updated [Paper plugin](../../../minecraft/map-control/README.md), configure `MINECRAFT_WORLD_SOCKET` to its absolute `world.sock` path, apply the embedded waypoint migration, and deploy the website through the normal workflow. Local implementation does not install the plugin or restart Minecraft. Unset the new environment variable to disable observations without disabling the base map or existing controls.

The [implementation plan](../../plans/2026-09-28-minecraft-explorer.md) records observed checks and remaining live verification. Pure Java fixtures cover protocol framing and saved-region decoding; Rust tests cover input/output bounds, socket behavior, and administrator guards; PostgreSQL tests cover persistence and database constraints; browser fixtures cover rendering, scanning, waypoints, and mobile layout.
