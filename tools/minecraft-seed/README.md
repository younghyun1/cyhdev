# Minecraft biome sampler

This Rust library samples Java 26.3 Overworld biomes using Pumpkin revision `4426d1113a211e6018a2db416e33b6b8a7802614`. It initializes only the biome noise router. It does not start a Minecraft server, read world files, or generate chunks. Default and large-biomes worlds use their respective noise routers; matching the configured preset is required.

The continuous map calls `Predictor::new()` once in the web backend and awaits `predict(request)` on its clonable handle. One dedicated CPU worker retains the seeded router and borrowing sampler across requests, rebuilding only when seed or preset changes. Its private single-thread Rayon pool prevents per-core density-buffer pools from multiplying retained memory. Eight queued jobs are permitted; excess admission returns `Error::Busy`. Dropping an awaiting future cancels queued work and active sampling checks cancellation between rows. All handles dropping closes the worker. The standalone executable remains available for compatibility with the old bounded prediction endpoint and offline fixtures.

Build from the workspace root with `cargo build --locked --package minecraft-seed`. This uses the development profile. The package requires Pumpkin's Rust 1.96 minimum and uses the repository's nightly toolchain. The executable is `target/debug/minecraft-seed` unless a Cargo target override changes its location.

The Docker builders set `RUST_MIN_STACK=33554432` because compiling Pumpkin's generated tables overflowed rustc's default stack on the pinned Alpine toolchain. This setting applies to compilation, not the deployed worker.

Send one JSON object followed by a newline to stdin, then close stdin. The seed travels only through this private pipe, never command arguments. Read one JSON response followed by a newline from stdout. A nonzero exit indicates failure; the process intentionally emits no diagnostic input or seed. The supervising backend must bound subprocess concurrency, cap stdout at 2 MiB, enforce a deadline, and kill and reap cancelled or timed-out processes.

```json
{"seed":1,"large_biomes":false,"y":64,"min_x":176,"min_z":148,"width":1,"height":1,"step":4}
```

```json
{"generator_revision":"pumpkin-4426d1113a211e6018a2db416e33b6b8a7802614-java26.3","large_biomes":false,"cells":[{"x":176,"z":148,"biome":"minecraft:stony_shore"}]}
```

Requests are at most 4096 bytes including their newline. `seed` is a signed 64-bit integer. Width and height are positive and their product is at most 4096. Step is positive, all sample X/Z coordinates are within ±30,000,000, and Y is within -64 through 319. Coordinates and step are in blocks. Sampling uses the containing raw quart biome, rounding negative coordinates toward negative infinity. The result is a fixed-Y slice, not the surface biome or a Voronoi-blended rendering. Cells are ordered by Z row, then X column.

The in-process path uses the same bounds without serializing a pipe request. A 64 by 64 tile fixes the largest density buffer at 16 KiB. Pumpkin's pool retains at most 1,024 such buffers on the owning CPU thread, so the backend reserves sampler and index headroom within the shared 512 MiB prediction budget. The tile cache accounts retained palettes and indices separately from bounded active requests.

The same vanilla biome decision tree serves both presets; large-biomes behavior comes from its distinct noise router. Aligned grids use Pumpkin's volume sampler, while arbitrary block grids sample their individually rounded quart coordinates. Each tree lookup starts without a previous leaf so a coordinate remains stable across overlapping viewports and request order. Vanilla retains the previous nearest leaf when two candidates have equal distance; this preview's deterministic choice can differ from saved biomes at those boundaries. Predictions are approximate and cannot represent player edits, previously generated terrain from older game versions, custom datapacks, or custom generators.

Independent saved Paper 26.3-28-dev `93c9b2a` fixtures using synthetic seed 1 contain 75,264 raw quart samples per preset, across 49 chunks and all 96 Overworld quart heights. The default fixture has 97 differences, all equal-distance beach/stony-shore choices. The large-biomes fixture has 96 differences, all equal-distance beach/plains choices and includes negative Z coordinates. Combined, 150,335 of 150,528 samples match. This validates the sampled areas and preset selection, not every biome, seed or coordinate. Scalar and volume climate values agree across 1024 additional sample points per preset; overlapping-grid and scalar prediction tests verify stable output.

Run focused checks with `cargo test --locked --package minecraft-seed`, `cargo clippy --locked --package minecraft-seed --all-targets`, and `cargo fmt --package minecraft-seed -- --check`. An ignored test accepts an independently extracted synthetic seed-1 fixture with TSV columns `quart_x`, `quart_y`, `quart_z`, and namespaced `biome`, with one header row:

```sh
MINECRAFT_SEED_FIXTURE=/path/to/default.tsv cargo test --locked --package minecraft-seed compares_external_vanilla_fixture -- --ignored --nocapture
MINECRAFT_SEED_FIXTURE=/path/to/large.tsv MINECRAFT_SEED_LARGE_BIOMES=1 cargo test --locked --package minecraft-seed compares_external_vanilla_fixture -- --ignored --nocapture
```

The fixture comparison reports every mismatch count and rejects any mismatch whose saved and predicted biome distances differ. Add `MINECRAFT_SEED_REQUIRE_EXACT=1` to require exact saved-palette equality; that stricter check fails for both fixtures above. Retain this distinction when replacing the pinned generator.

Keep real seeds and world fixtures outside Git. This package is GPL-3.0-only; see [LICENSE](LICENSE) and [third-party notices](THIRD_PARTY_NOTICES.md).
