package com.cyhdev.minecraft;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.nio.channels.Channels;
import java.nio.charset.StandardCharsets;
import java.util.List;

/** Protocol tests run without a Minecraft server or Bukkit initialization. */
public final class WorldProtocolTest {
    private static int checks;

    private WorldProtocolTest() {}

    public static void main(String[] args) throws Exception {
        acceptsCatalogAndSlices();
        rejectsMalformedAndAmbiguousJson();
        limitsAreaAndCoordinates();
        limitsVerticalRanges();
        limitsFramesAndEncoding();
        preservesResponseNullsAndBounds();
        System.out.println("WorldProtocolTest: " + checks + " checks passed");
    }

    private static void acceptsCatalogAndSlices() throws IOException {
        check(WorldProtocol.parse("{\"kind\":\"catalog\"}").world() == null, "Catalog world");
        check(WorldProtocol.parse(area(0, 0, 8, 8, "null")).y() == null, "Surface biome");
        check(WorldProtocol.parse(area(-8, -8, 8, 8, "-64")).y() == -64, "Underground biome");
        check(WorldProtocol.parse(area(0, 0, 1, 1, "null").replace(",\"y\":null", "")).y() == null, "Omitted slice");
        check(WorldProtocol.parse(blocks(4, 4, -64, 447)).max_y() == 447, "Inclusive 512 block height range");
    }

    private static void rejectsMalformedAndAmbiguousJson() throws IOException {
        reject("{\"kind\":\"catalog\",\"kind\":\"catalog\"}");
        reject("{\"kind\":\"catalog\",\"world\":\"minecraft:overworld\"}");
        reject("{\"kind\":\"save\"}");
        reject("{kind:'catalog'}");
        reject("{\"kind\":\"catalog\",}");
        reject("{\"kind\":\"catalog\"}{}");
        reject("[]");
        reject("null");
        reject(area(0, 0, 1, 1, "null").replace("\"width\":1", "\"width\":1.5"));
        reject(area(0, 0, 1, 1, "null").replace("\"width\":1", "\"width\":\"1\""));
        reject(area(0, 0, 1, 1, "null").replace("\"width\":1", "\"width\":1e0"));
        reject(area(0, 0, 1, 1, "null").replace("\"width\":1", "\"width\":null"));
        reject(area(0, 0, 1, 1, "null").replace("minecraft:overworld", "overworld"));
        reject(area(0, 0, 1, 1, "null").replace("minecraft:overworld", "Minecraft:overworld"));
        reject(area(0, 0, 1, 1, "null").replace("minecraft:overworld", "m:" + "a".repeat(127)));
        reject(blocks(1, 1, 0, 1).replace("minecraft:diamond_ore", "diamond_ore"));
    }

    private static void limitsAreaAndCoordinates() throws IOException {
        reject(area(0, 0, 9, 1, "null"));
        reject(area(0, 0, 1, 9, "null"));
        reject(area(0, 0, 0, 1, "null"));
        reject(area(0, 0, 1, -1, "null"));
        reject(blocks(5, 1, 0, 1));
        reject(blocks(1, 5, 0, 1));
        check(WorldProtocol.parse(area(-1_875_000, 1_874_999, 1, 1, "null")).width() == 1, "World border accepted");
        reject(area(-1_875_001, 0, 1, 1, "null"));
        reject(area(1_875_000, 0, 1, 1, "null"));
        reject(area(1_874_999, 0, 2, 1, "null"));
        reject(area(Integer.MAX_VALUE, 0, 1, 1, "null"));
        reject(area(Integer.MIN_VALUE, 0, 1, 1, "null"));
        reject(area(0, 0, 1, 1, "null").replace("\"chunk_x\":0", "\"chunk_x\":2147483648"));
    }

    private static void limitsVerticalRanges() throws IOException {
        check(WorldProtocol.parse(area(0, 0, 1, 1, "-2032")).y() == -2032, "Minimum Y accepted");
        check(WorldProtocol.parse(area(0, 0, 1, 1, "2031")).y() == 2031, "Maximum Y accepted");
        check(WorldProtocol.parse(blocks(1, 1, 0, 0)).min_y() == 0, "Single-layer block query");
        reject(area(0, 0, 1, 1, "2032"));
        reject(area(0, 0, 1, 1, "-2033"));
        reject(blocks(1, 1, -64, 448));
        reject(blocks(1, 1, 10, 9));
        reject(blocks(1, 1, -2033, -2032));
        reject(blocks(1, 1, 2031, 2032));
    }

    private static void limitsFramesAndEncoding() throws IOException {
        check(read("{\"kind\":\"catalog\"}\n".getBytes(StandardCharsets.UTF_8)).equals("{\"kind\":\"catalog\"}"), "Newline frame");
        check(read((" ".repeat(WorldProtocol.REQUEST_BYTES - 1) + "\n").getBytes(StandardCharsets.UTF_8)).length() == WorldProtocol.REQUEST_BYTES - 1, "Maximum frame");
        rejectFrame("a".repeat(WorldProtocol.REQUEST_BYTES).getBytes(StandardCharsets.UTF_8));
        rejectFrame("{\"kind\":\"catalog\"}".getBytes(StandardCharsets.UTF_8));
        rejectFrame("{}\n{}\n".getBytes(StandardCharsets.UTF_8));
        rejectFrame(new byte[] {(byte) 0xc3, (byte) 0x28, (byte) '\n'});
    }

    private static void preservesResponseNullsAndBounds() throws IOException {
        var response = new WorldProtocol.Response("catalog", null, 1L, 0, 0, false, List.of(), List.of(), List.of(), List.of(), List.of());
        String encoded = StandardCharsets.UTF_8.decode(WorldProtocol.encode(response)).toString();
        check(encoded.contains("\"world\":null"), "Null world included");
        check(encoded.contains("\"structures\":[]"), "Empty structure array included");
        check(encoded.endsWith("\n"), "Response frame terminator");
        var oversized = new WorldProtocol.Response("catalog", null, 1L, 0, 0, false, List.of(), List.of("a".repeat(WorldProtocol.RESPONSE_BYTES)), List.of(), List.of(), List.of());
        boolean rejected = false;
        try { WorldProtocol.encode(oversized); } catch (IOException expected) { rejected = true; }
        check(rejected, "Response size limit");
    }

    private static String area(int x, int z, int width, int height, String y) {
        return "{\"kind\":\"area\",\"world\":\"minecraft:overworld\",\"chunk_x\":" + x + ",\"chunk_z\":" + z
            + ",\"width\":" + width + ",\"height\":" + height + ",\"y\":" + y + "}";
    }

    private static String blocks(int width, int height, int min, int max) {
        return "{\"kind\":\"blocks\",\"world\":\"minecraft:overworld\",\"chunk_x\":0,\"chunk_z\":0,\"width\":" + width
            + ",\"height\":" + height + ",\"block\":\"minecraft:diamond_ore\",\"min_y\":" + min + ",\"max_y\":" + max + "}";
    }

    private static void reject(String input) throws IOException {
        boolean rejected = false;
        try { WorldProtocol.parse(input); } catch (IOException expected) { rejected = true; }
        check(rejected, "Invalid request must be rejected: " + input);
    }

    private static String read(byte[] bytes) throws IOException {
        return WorldProtocol.read(Channels.newChannel(new ByteArrayInputStream(bytes)));
    }

    private static void rejectFrame(byte[] bytes) throws IOException {
        boolean rejected = false;
        try { read(bytes); } catch (IOException expected) { rejected = true; }
        check(rejected, "Invalid frame must be rejected");
    }

    private static void check(boolean condition, String message) {
        checks++;
        if (!condition) throw new AssertionError(message);
    }
}
