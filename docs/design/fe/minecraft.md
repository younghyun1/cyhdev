# Minecraft world atlas

The public Minecraft page uses squaremap terrain with a native Leaflet client. Its scoped palette, beveled controls, square outlines, monospace labels, and pixelated terrain evoke Minecraft without importing another component framework or game assets. Desktop tools occupy a scrollable left panel; mobile tools sit below a fixed-height map with their own scrolling. Both fit between the measured website bars. Native inputs, visible keyboard focus, coordinate navigation, text legends, and text result lists keep the controls usable without relying on map colors or pointer interaction alone.

The Layers panel surveys a selected 128 by 128 block area, shows biome colors or surface elevation, filters sampled biomes, and lists saved structure starts. A Y selector samples cave biomes while elevation continues to describe the surface. Gold outlines identify completed surveys; the next survey remains separately outlined. Coverage counts, time, and incomplete results stay visible. Structure bounds describe saved metadata and can outlast a demolished structure.

The Blocks panel searches a selected 64 by 64 block area and an inclusive vertical range of at most 512 layers. Cyan markers and a result list identify exact matches; results stop at 512 and report truncation. Dimension changes discard old observations and ignore late responses. World queries are explicit; terrain and published player snapshots refresh separately while visible. Base terrain does not wait for the world-analysis catalog.

Places are public shared waypoints. Administrator controls create, edit, and delete them; the backend independently checks current authority. A location link preserves the selected dimension and coordinates. Travel tools provide coordinate navigation, spawn navigation, a horizontal distance ruler, and Overworld/Nether coordinate conversion. The conversion estimates a portal location, not an established link.

The original map button opens the existing squaremap document in its opaque-origin sandbox. The native explorer consumes bounded JSON and PNG data and uses text nodes for all Leaflet labels. Plugin catalog failure leaves terrain and coordinate tools available; waypoint editing and analysis remain unavailable until their world metadata loads.

See the [backend design](../../architecture/be/minecraft-explorer.md) for limits and data provenance and the [implementation plan](../../plans/2026-09-28-minecraft-explorer.md) for observed verification and activation.
