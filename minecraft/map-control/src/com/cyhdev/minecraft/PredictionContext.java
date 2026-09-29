package com.cyhdev.minecraft;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.SecureRandom;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import net.minecraft.world.level.biome.MultiNoiseBiomeSourceParameterLists;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.storage.LevelResource;
import org.bukkit.Bukkit;
import org.bukkit.World;
import org.bukkit.craftbukkit.CraftWorld;
import xyz.jpenilla.squaremap.api.SquaremapProvider;
import xyz.jpenilla.squaremap.api.WorldIdentifier;
import xyz.jpenilla.squaremap.common.data.MapWorldInternal;

/** Version-locked read-only adapter. Bukkit does not expose effective presets or pending chunk holders. */
final class PredictionContext {
    private final byte[] revisionKey = new byte[32];

    PredictionContext() { new SecureRandom().nextBytes(revisionKey); }

    static final class Unsupported extends IOException {
        private static final long serialVersionUID = 1L;
        Unsupported() { super("Unsupported prediction profile"); }
    }

    record Snapshot(String world, String worldId, long seed, String preset, String revision,
                    Path regionDirectory, PredictionPacks.Selection packs, List<String> states) {
        @Override public String toString() { return "PredictionSnapshot[private profile]"; }
    }

    /** Call only on the main thread. The holder lookup does not create tickets or request chunk data. */
    Snapshot capture(World world, WorldProtocol.Request request) throws Unsupported {
        try {
            if (!Bukkit.getMinecraftVersion().equals("26.3") || !(world instanceof CraftWorld craft)
                    || !world.key().asString().equals("minecraft:overworld")
                    || world.getEnvironment() != World.Environment.NORMAL || world.getMinHeight() != -64 || world.getMaxHeight() != 320
                    || world.getGenerator() != null || world.getBiomeProvider() != null) throw new Unsupported();
            var enabledPacks = Bukkit.getServer().getDatapackManager().getEnabledPacks();
            if (enabledPacks.isEmpty() || enabledPacks.size() > PredictionPacks.MAX_PACKS) throw new Unsupported();
            List<String> packNames = new ArrayList<>();
            for (var pack : enabledPacks) {
                if (!PredictionPacks.supportedName(pack.getName())) throw new Unsupported();
                packNames.add(pack.getName());
            }
            if (!packNames.contains("vanilla")) throw new Unsupported();
            packNames.sort(String::compareTo);
            var level = craft.getHandle();
            var generator = level.getChunkSource().getGenerator();
            if (!(generator instanceof NoiseBasedChunkGenerator noise)
                    || !(noise.getBiomeSource() instanceof MultiNoiseBiomeSource biomes)
                    || !biomes.stable(MultiNoiseBiomeSourceParameterLists.OVERWORLD)) throw new Unsupported();
            String preset;
            if (noise.stable(NoiseGeneratorSettings.OVERWORLD)) preset = "default";
            else if (noise.stable(NoiseGeneratorSettings.LARGE_BIOMES)) preset = "large_biomes";
            else throw new Unsupported();
            var mapped = SquaremapProvider.get().getWorldIfEnabled(WorldIdentifier.parse(request.world()));
            if (mapped.isEmpty() || !(mapped.get() instanceof MapWorldInternal mapWorld)) throw new Unsupported();
            var limits = mapWorld.visibilityLimit();
            if (limits.getShapes().size() > 64) throw new Unsupported();
            var holders = level.moonrise$getChunkTaskScheduler().chunkHolderManager;
            List<String> states = new ArrayList<>(request.width() * request.height());
            for (int dz = 0; dz < request.height(); dz++) {
                for (int dx = 0; dx < request.width(); dx++) {
                    int x = request.chunk_x() + dx;
                    int z = request.chunk_z() + dz;
                    boolean visible = true;
                    for (int blockZ = z * 16; visible && blockZ < z * 16 + 16; blockZ++) {
                        for (int blockX = x * 16; blockX < x * 16 + 16; blockX++) {
                            if (!limits.shouldRenderColumn(blockX, blockZ)
                                    || !level.getWorldBorder().isWithinBounds(blockX, blockZ)) { visible = false; break; }
                        }
                    }
                    if (!visible) { states.add("excluded"); continue; }
                    var holder = holders.getChunkHolder(x, z);
                    if (holder == null) { states.add("absent"); continue; }
                    var chunk = holder.getCurrentChunk();
                    states.add(chunk != null && chunk.getPersistedStatus() == ChunkStatus.FULL ? "generated" : "unknown");
                }
            }
            long seed = level.getSeed();
            String worldId = world.getUID().toString();
            // The keyed digest prevents the public revision from becoming a seed dictionary oracle.
            String revision = revision(worldId + "\n" + seed + "\n" + preset + "\n" + Bukkit.getVersion()
                + "\n" + String.join(",", packNames) + "\n" + System.identityHashCode(noise.generatorSettings().value())
                + "\n" + System.identityHashCode(biomes) + "\n" + System.identityHashCode(level.getChunkSource().randomState())
                + "\n" + System.identityHashCode(level.getServer().getResourceManager()));
            var packs = new PredictionPacks.Selection(level.getServer().getWorldPath(LevelResource.DATAPACK_DIR), List.copyOf(packNames));
            return new Snapshot(request.world(), worldId, seed, preset, revision, world.getWorldPath().resolve("region"), packs, List.copyOf(states));
        } catch (LinkageError | RuntimeException error) {
            // An incompatible Paper/squaremap implementation must disable predictions, not guess its state.
            throw new Unsupported();
        }
    }

    String verifiedRevision(Snapshot snapshot, String packsFingerprint) throws Unsupported {
        return revision(snapshot.revision() + "\n" + packsFingerprint);
    }

    private String revision(String input) throws Unsupported {
        try {
            Mac mac = Mac.getInstance("HmacSHA256");
            mac.init(new SecretKeySpec(revisionKey, "HmacSHA256"));
            return HexFormat.of().formatHex(mac.doFinal(input.getBytes(StandardCharsets.UTF_8)));
        } catch (GeneralSecurityException error) { throw new Unsupported(); }
    }

    static List<WorldProtocol.Coverage> combine(WorldProtocol.Request request, Snapshot before, Snapshot after,
                                               List<Boolean> diskAbsent) throws Unsupported {
        if (!before.worldId().equals(after.worldId()) || !before.revision().equals(after.revision())
                || before.seed() != after.seed() || !before.preset().equals(after.preset())) throw new Unsupported();
        List<WorldProtocol.Coverage> result = new ArrayList<>(diskAbsent.size());
        for (int i = 0; i < diskAbsent.size(); i++) {
            String first = before.states().get(i);
            String second = after.states().get(i);
            String state;
            if (first.equals("excluded") || second.equals("excluded")) state = "excluded";
            else if (first.equals("generated") || second.equals("generated")) state = "generated";
            else if (first.equals("absent") && second.equals("absent") && diskAbsent.get(i)) state = "ungenerated";
            else state = "unknown";
            result.add(new WorldProtocol.Coverage(request.chunk_x() + i % request.width(), request.chunk_z() + i / request.width(), state));
        }
        return List.copyOf(result);
    }
}
