import { describe, expect, it } from "vitest";
import {
  decodeServerEventFrame,
  encodePingFrame,
  encodeSendMessageFrame,
  encodeTypingFrame,
} from "../services/live_chat_binary";

const CLIENT_MESSAGE_ID = "018f3f7d-5a76-7d8f-8123-456789abcdef";
const MESSAGE_ID = "018f3f7d-5a76-7d8f-8123-456789abc001";
describe("live chat binary client frames", () => {
  it("encodes send_message as type byte, raw UUID, and UTF-8 body", () => {
    const frame = bytes(encodeSendMessageFrame(CLIENT_MESSAGE_ID, "hello"));

    expect(frame[0]).toBe(0x01);
    expect(hex(frame.slice(1, 17))).toBe(CLIENT_MESSAGE_ID.replaceAll("-", ""));
    expect(frame.slice(17, 19)).toEqual([0, 5]);
    expect(ascii(frame.slice(19))).toBe("hello");
  });

  it("encodes typing start/stop as one-byte frames", () => {
    expect(bytes(encodeTypingFrame(true))).toEqual([0x02]);
    expect(bytes(encodeTypingFrame(false))).toEqual([0x03]);
  });

  it("encodes ping nonce as big-endian u64", () => {
    expect(bytes(encodePingFrame(42n))).toEqual([0x04, 0, 0, 0, 0, 0, 0, 0, 42]);
  });
});

describe("live chat binary server frames", () => {
  it("decodes presence and error frames", () => {
    expect(decodeServerEventFrame(buffer([0x85, 0, 0, 2, 1]))).toEqual({
      type: "presence",
      connected_count: 513,
    });

    expect(
      decodeServerEventFrame(
        buffer([
          0x87,
          0,
          3,
          ...utf8("bad"),
          0,
          4,
          ...utf8("nope"),
        ]),
      ),
    ).toEqual({
      type: "error",
      code: "bad",
      message: "nope",
    });
  });

  it("decodes typing_set with binary IPv4 guest actor", () => {
    const expiresAtMs = 1_700_000_000_000;
    const frame = [
      0x84,
      ...i64(expiresAtMs),
      1,
      ...guestActorBytes("203.0.113.9", "guest@203.0.113.9", "🇺🇸"),
    ];

    expect(decodeServerEventFrame(buffer(frame))).toEqual({
      type: "typing_set",
      expires_at: new Date(expiresAtMs).toISOString(),
      actors: [
        {
          actor_key: { type: "guest", value: "203.0.113.9" },
          sender_kind: 2,
          user_id: null,
          guest_ip: "203.0.113.9",
          display_name: "guest@203.0.113.9",
          country_flag: "🇺🇸",
          user_profile_picture_url: null,
        },
      ],
    });
  });

  it("decodes message_ack with binary IPv6 guest IP and edited/deleted timestamps", () => {
    const createdAtMs = 1_700_000_000_000;
    const editedAtMs = 1_700_000_001_000;
    const deletedAtMs = 1_700_000_002_000;
    const frame = [
      0x83,
      ...uuidBytes(CLIENT_MESSAGE_ID),
      ...messageBytes({
        guestIpKind: "ipv6",
        guestIpText: "2001:db8:0:0:0:0:0:1",
        guestIpBytes: [
          0x20,
          0x01,
          0x0d,
          0xb8,
          0,
          0,
          0,
          0,
          0,
          0,
          0,
          0,
          0,
          0,
          0,
          1,
        ],
        createdAtMs,
        editedAtMs,
        deletedAtMs,
      }),
    ];

    expect(decodeServerEventFrame(buffer(frame))).toEqual({
      type: "message_ack",
      client_message_id: CLIENT_MESSAGE_ID,
      message: {
        live_chat_message_id: MESSAGE_ID,
        room_key: "main",
        user_id: null,
        guest_ip: "2001:db8:0:0:0:0:0:1",
        sender_kind: 2,
        sender_display_name: "guest@2001:db8:0:0:0:0:0:1",
        sender_country_flag: "🇺🇸",
        user_profile_picture_url: null,
        message_body: "hello",
        message_created_at: new Date(createdAtMs).toISOString(),
        message_edited_at: new Date(editedAtMs).toISOString(),
        message_deleted_at: new Date(deletedAtMs).toISOString(),
      },
    });
  });

  it("returns null for malformed/trailing frames", () => {
    expect(decodeServerEventFrame(buffer([0xff]))).toBeNull();
    expect(decodeServerEventFrame(buffer([0x85, 0, 0]))).toBeNull();
    expect(decodeServerEventFrame(buffer([0x85, 0, 0, 0, 1, 0]))).toBeNull();
  });
});

function messageBytes(input: {
  guestIpKind: "ipv4" | "ipv6";
  guestIpText: string;
  guestIpBytes: number[];
  createdAtMs: number;
  editedAtMs: number;
  deletedAtMs: number;
}): number[] {
  return [
    ...uuidBytes(MESSAGE_ID),
    ...str("main"),
    0x02,
    input.guestIpKind === "ipv4" ? 0x04 : 0x06,
    ...input.guestIpBytes,
    2,
    ...str(`guest@${input.guestIpText}`),
    ...str("🇺🇸"),
    0xff,
    0xff,
    ...i64(input.createdAtMs),
    0x03,
    ...i64(input.editedAtMs),
    ...i64(input.deletedAtMs),
    ...str("hello"),
  ];
}

function guestActorBytes(ip: string, displayName: string, flag: string): number[] {
  return [
    0x02,
    0x04,
    ...ip.split(".").map((part) => Number(part)),
    2,
    ...str(displayName),
    ...str(flag),
    0xff,
    0xff,
  ];
}

function uuidBytes(uuid: string): number[] {
  const normalized = uuid.replaceAll("-", "");
  const result: number[] = [];
  for (let i = 0; i < normalized.length; i += 2) {
    result.push(Number.parseInt(normalized.slice(i, i + 2), 16));
  }
  return result;
}

function str(value: string): number[] {
  const valueBytes = utf8(value);
  return [(valueBytes.length >> 8) & 0xff, valueBytes.length & 0xff, ...valueBytes];
}

function utf8(value: string): number[] {
  return Array.from(new TextEncoder().encode(value));
}

function i64(value: number): number[] {
  const output = new Array<number>(8).fill(0);
  let next = BigInt(value);
  for (let i = 7; i >= 0; i -= 1) {
    output[i] = Number(next & 0xffn);
    next >>= 8n;
  }
  return output;
}

function buffer(values: number[]): ArrayBuffer {
  return new Uint8Array(values).buffer;
}

function bytes(value: ArrayBuffer): number[] {
  return Array.from(new Uint8Array(value));
}

function hex(values: number[]): string {
  return values.map((value) => value.toString(16).padStart(2, "0")).join("");
}

function ascii(values: number[]): string {
  return new TextDecoder().decode(new Uint8Array(values));
}
