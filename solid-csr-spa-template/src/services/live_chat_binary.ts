import type {
  ChatActor,
  ChatActorKey,
  LiveChatMessageItem,
  LiveChatServerEvent,
  RtcParticipant,
  RtcPeerPhase,
} from "../dtos/responses/live_chat";

export const LIVE_CHAT_BINARY_PROTOCOL = "livechat.bin.v1";

const CLIENT_SEND_MESSAGE = 0x01;
const CLIENT_TYPING_START = 0x02;
const CLIENT_TYPING_STOP = 0x03;
const CLIENT_PING = 0x04;
const CLIENT_RTC = 0x05;

const SERVER_HELLO = 0x81;
const SERVER_MESSAGE = 0x82;
const SERVER_MESSAGE_ACK = 0x83;
const SERVER_TYPING_SET = 0x84;
const SERVER_PRESENCE = 0x85;
const SERVER_PONG = 0x86;
const SERVER_ERROR = 0x87;
const SERVER_RTC = 0x90;

// RTC client sub-opcodes (second byte after CLIENT_RTC).
const RTC_C_JOIN = 0x01;
const RTC_C_ANSWER = 0x02;
const RTC_C_ICE = 0x03;
const RTC_C_LEAVE = 0x04;
const RTC_C_MEDIA_STATE = 0x05;

// RTC server sub-opcodes (second byte after SERVER_RTC).
const RTC_S_ANSWER = 0x01;
const RTC_S_OFFER = 0x02;
const RTC_S_ICE = 0x03;
const RTC_S_PEER_STATE = 0x04;
const RTC_S_ROSTER = 0x05;
const RTC_S_ERROR = 0x06;

const RTC_PHASE_LEFT = 0x00;

const ACTOR_USER = 0x01;
const ACTOR_GUEST = 0x02;

const IP_NONE = 0x00;
const IP_V4 = 0x04;
const IP_V6 = 0x06;

const NONE_STRING_LEN = 0xffff;
const MESSAGE_FLAG_EDITED_AT = 0x01;
const MESSAGE_FLAG_DELETED_AT = 0x02;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/// Format an IPv6 address the way Rust's `Ipv6Addr` Display does: IPv4-mapped
/// addresses as `::ffff:a.b.c.d`, otherwise the longest run (>= 2) of zero
/// hextets compressed to `::` (leftmost on ties), hextets in lowercase hex
/// without leading zeros. The SFU stamps MediaStream ids with that exact form
/// (`actor_stream_id`), so any deviation breaks actor-to-stream mapping and
/// the participant's tile never binds its stream.
export function formatIpv6Canonical(hextets: readonly number[]): string {
  const seg = (i: number): number => hextets[i] ?? 0;
  const isV4Mapped =
    seg(0) === 0 &&
    seg(1) === 0 &&
    seg(2) === 0 &&
    seg(3) === 0 &&
    seg(4) === 0 &&
    seg(5) === 0xffff;
  if (isV4Mapped) {
    const g = seg(6);
    const h = seg(7);
    return `::ffff:${(g >> 8) & 0xff}.${g & 0xff}.${(h >> 8) & 0xff}.${h & 0xff}`;
  }
  let bestStart = -1;
  let bestLen = 0;
  let runStart = -1;
  let runLen = 0;
  for (let i = 0; i < 8; i += 1) {
    if (seg(i) === 0) {
      if (runStart === -1) runStart = i;
      runLen += 1;
      if (runLen > bestLen) {
        bestLen = runLen;
        bestStart = runStart;
      }
    } else {
      runStart = -1;
      runLen = 0;
    }
  }
  const hex = (i: number): string => seg(i).toString(16);
  if (bestLen < 2) {
    return Array.from({ length: 8 }, (_, i) => hex(i)).join(":");
  }
  const head = Array.from({ length: bestStart }, (_, i) => hex(i)).join(":");
  const tailStart = bestStart + bestLen;
  const tail = Array.from({ length: 8 - tailStart }, (_, i) =>
    hex(tailStart + i),
  ).join(":");
  return `${head}::${tail}`;
}

export function encodeSendMessageFrame(
  clientMessageId: string,
  body: string,
): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(CLIENT_SEND_MESSAGE);
  writer.writeUuid(clientMessageId);
  writer.writeString(body);
  return writer.finish();
}

export function encodeTypingFrame(isTyping: boolean): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(isTyping ? CLIENT_TYPING_START : CLIENT_TYPING_STOP);
  return writer.finish();
}

export function encodePingFrame(nonce: bigint): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(CLIENT_PING);
  writer.writeU64(nonce);
  return writer.finish();
}

export function encodeRtcJoinFrame(
  sdp: string,
  wantAudio: boolean,
  wantVideo: boolean,
): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(CLIENT_RTC);
  writer.writeU8(RTC_C_JOIN);
  writer.writeU8(wantAudio ? 1 : 0);
  writer.writeU8(wantVideo ? 1 : 0);
  writer.writeString(sdp);
  return writer.finish();
}

export function encodeRtcAnswerFrame(sdp: string): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(CLIENT_RTC);
  writer.writeU8(RTC_C_ANSWER);
  writer.writeString(sdp);
  return writer.finish();
}

export function encodeRtcIceFrame(
  candidate: string,
  sdpMid: string | null,
  sdpMlineIndex: number | null,
): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(CLIENT_RTC);
  writer.writeU8(RTC_C_ICE);
  writer.writeString(candidate);
  if (sdpMid !== null) {
    writer.writeU8(1);
    writer.writeString(sdpMid);
  } else {
    writer.writeU8(0);
  }
  if (sdpMlineIndex !== null) {
    writer.writeU8(1);
    writer.writeU16(sdpMlineIndex);
  } else {
    writer.writeU8(0);
  }
  return writer.finish();
}

export function encodeRtcLeaveFrame(): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(CLIENT_RTC);
  writer.writeU8(RTC_C_LEAVE);
  return writer.finish();
}

export function encodeRtcMediaStateFrame(
  micOn: boolean,
  camOn: boolean,
): ArrayBuffer {
  const writer = new BinaryWriter();
  writer.writeU8(CLIENT_RTC);
  writer.writeU8(RTC_C_MEDIA_STATE);
  writer.writeU8(micOn ? 1 : 0);
  writer.writeU8(camOn ? 1 : 0);
  return writer.finish();
}

export function decodeServerEventFrame(buffer: ArrayBuffer): LiveChatServerEvent | null {
  try {
    const reader = new BinaryReader(buffer);
    const eventType = reader.readU8();
    switch (eventType) {
      case SERVER_HELLO: {
        const actor = reader.readActor();
        const connectedCount = reader.readU32();
        const recentCount = reader.readU16();
        const recentMessages: LiveChatMessageItem[] = [];
        for (let i = 0; i < recentCount; i += 1) {
          recentMessages.push(reader.readMessage());
        }
        reader.finish();
        return {
          type: "hello",
          actor,
          recent_messages: recentMessages,
          connected_count: connectedCount,
        };
      }
      case SERVER_MESSAGE: {
        const message = reader.readMessage();
        reader.finish();
        return { type: "message", message };
      }
      case SERVER_MESSAGE_ACK: {
        const clientMessageId = reader.readUuid();
        const message = reader.readMessage();
        reader.finish();
        return {
          type: "message_ack",
          client_message_id: clientMessageId,
          message,
        };
      }
      case SERVER_TYPING_SET: {
        const expiresAt = reader.readTime();
        const actorCount = reader.readU8();
        const actors: ChatActor[] = [];
        for (let i = 0; i < actorCount; i += 1) {
          actors.push(reader.readActor());
        }
        reader.finish();
        return { type: "typing_set", actors, expires_at: expiresAt };
      }
      case SERVER_PRESENCE: {
        const connectedCount = reader.readU32();
        reader.finish();
        return { type: "presence", connected_count: connectedCount };
      }
      case SERVER_PONG: {
        const nonce = reader.readU64().toString();
        reader.finish();
        return { type: "heartbeat_ack", nonce };
      }
      case SERVER_ERROR: {
        const code = reader.readString();
        const message = reader.readString();
        reader.finish();
        return { type: "error", code, message };
      }
      case SERVER_RTC: {
        const event = reader.readRtcServerSignal();
        reader.finish();
        return event;
      }
      default:
        return null;
    }
  } catch (err) {
    console.error("Failed to parse binary live chat event:", err);
    return null;
  }
}

class BinaryWriter {
  private bytes: number[] = [];

  finish(): ArrayBuffer {
    return new Uint8Array(this.bytes).buffer;
  }

  writeU8(value: number) {
    this.bytes.push(value & 0xff);
  }

  writeU16(value: number) {
    this.bytes.push((value >> 8) & 0xff, value & 0xff);
  }

  writeU64(value: bigint) {
    for (let shift = 56n; shift >= 0n; shift -= 8n) {
      this.bytes.push(Number((value >> shift) & 0xffn));
    }
  }

  writeUuid(value: string) {
    const normalized = value.replaceAll("-", "");
    if (normalized.length !== 32) {
      throw new Error("Invalid UUID length");
    }
    for (let i = 0; i < normalized.length; i += 2) {
      this.bytes.push(Number.parseInt(normalized.slice(i, i + 2), 16));
    }
  }

  writeString(value: string) {
    const bytes = encoder.encode(value);
    if (bytes.length >= NONE_STRING_LEN) {
      throw new Error("String too long for live chat binary frame");
    }
    this.writeU16(bytes.length);
    for (const byte of bytes) {
      this.bytes.push(byte);
    }
  }
}

class BinaryReader {
  private readonly view: DataView;
  private offset = 0;

  constructor(buffer: ArrayBuffer) {
    this.view = new DataView(buffer);
  }

  finish() {
    if (this.offset !== this.view.byteLength) {
      throw new Error("Trailing bytes in live chat binary frame");
    }
  }

  readU8(): number {
    this.ensure(1);
    const value = this.view.getUint8(this.offset);
    this.offset += 1;
    return value;
  }

  readU16(): number {
    this.ensure(2);
    const value = this.view.getUint16(this.offset, false);
    this.offset += 2;
    return value;
  }

  readU32(): number {
    this.ensure(4);
    const value = this.view.getUint32(this.offset, false);
    this.offset += 4;
    return value;
  }

  readU64(): bigint {
    this.ensure(8);
    const value = this.view.getBigUint64(this.offset, false);
    this.offset += 8;
    return value;
  }

  readI64(): bigint {
    this.ensure(8);
    const value = this.view.getBigInt64(this.offset, false);
    this.offset += 8;
    return value;
  }

  readUuid(): string {
    const bytes = this.readBytes(16);
    const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
  }

  readString(): string {
    const len = this.readU16();
    if (len === NONE_STRING_LEN) {
      throw new Error("Unexpected null string in live chat binary frame");
    }
    return decoder.decode(this.readBytes(len));
  }

  readOptionalString(): string | null {
    const len = this.readU16();
    if (len === NONE_STRING_LEN) return null;
    return decoder.decode(this.readBytes(len));
  }

  readTime(): string {
    return new Date(Number(this.readI64())).toISOString();
  }

  readIp(): string | null {
    const family = this.readU8();
    if (family === IP_NONE) return null;
    if (family === IP_V4) {
      return Array.from(this.readBytes(4), (byte) => String(byte)).join(".");
    }
    if (family === IP_V6) {
      const bytes = this.readBytes(16);
      const hextets: number[] = [];
      for (let i = 0; i < bytes.length; i += 2) {
        hextets.push(((bytes[i] ?? 0) << 8) | (bytes[i + 1] ?? 0));
      }
      return formatIpv6Canonical(hextets);
    }
    throw new Error("Unknown IP family in live chat binary frame");
  }

  readActor(): ChatActor {
    const actorKind = this.readU8();
    let actorKey: ChatActorKey;
    let userId: string | null = null;
    let guestIp: string | null = null;

    if (actorKind === ACTOR_USER) {
      userId = this.readUuid();
      this.readIp();
      actorKey = { type: "user", value: userId };
    } else if (actorKind === ACTOR_GUEST) {
      guestIp = this.readIp();
      actorKey = { type: "guest", value: guestIp ?? "" };
    } else {
      throw new Error("Unknown actor kind in live chat binary frame");
    }

    return {
      actor_key: actorKey,
      sender_kind: this.readU8(),
      user_id: userId,
      guest_ip: guestIp,
      display_name: this.readString(),
      country_flag: this.readOptionalString(),
      user_profile_picture_url: this.readOptionalString(),
    };
  }

  readMessage(): LiveChatMessageItem {
    const liveChatMessageId = this.readUuid();
    const roomKey = this.readString();
    const actorKind = this.readU8();
    let userId: string | null = null;
    let guestIp: string | null = null;

    if (actorKind === ACTOR_USER) {
      userId = this.readUuid();
      this.readIp();
    } else if (actorKind === ACTOR_GUEST) {
      guestIp = this.readIp();
    } else {
      throw new Error("Unknown message actor kind in live chat binary frame");
    }

    const senderKind = this.readU8();
    const senderDisplayName = this.readString();
    const senderCountryFlag = this.readOptionalString();
    const userProfilePictureUrl = this.readOptionalString();
    const messageCreatedAt = this.readTime();
    const flags = this.readU8();
    const messageEditedAt =
      (flags & MESSAGE_FLAG_EDITED_AT) !== 0 ? this.readTime() : null;
    const messageDeletedAt =
      (flags & MESSAGE_FLAG_DELETED_AT) !== 0 ? this.readTime() : null;
    const messageBody = this.readString();

    return {
      live_chat_message_id: liveChatMessageId,
      room_key: roomKey,
      user_id: userId,
      guest_ip: guestIp,
      sender_kind: senderKind,
      sender_display_name: senderDisplayName,
      sender_country_flag: senderCountryFlag,
      user_profile_picture_url: userProfilePictureUrl,
      message_body: messageBody,
      message_created_at: messageCreatedAt,
      message_edited_at: messageEditedAt,
      message_deleted_at: messageDeletedAt,
    };
  }

  readRtcServerSignal(): LiveChatServerEvent {
    const subOp = this.readU8();
    switch (subOp) {
      case RTC_S_ANSWER:
        return { type: "rtc", kind: "answer", sdp: this.readString() };
      case RTC_S_OFFER:
        return { type: "rtc", kind: "offer", sdp: this.readString() };
      case RTC_S_ICE: {
        const candidate = this.readString();
        const sdpMid = this.readU8() !== 0 ? this.readString() : null;
        const sdpMlineIndex = this.readU8() !== 0 ? this.readU16() : null;
        return {
          type: "rtc",
          kind: "ice",
          candidate,
          sdp_mid: sdpMid,
          sdp_mline_index: sdpMlineIndex,
        };
      }
      case RTC_S_PEER_STATE: {
        const actor = this.readActor();
        const phase: RtcPeerPhase =
          this.readU8() === RTC_PHASE_LEFT ? "left" : "joined";
        const micOn = this.readU8() !== 0;
        const camOn = this.readU8() !== 0;
        return {
          type: "rtc",
          kind: "peer_state",
          actor,
          phase,
          mic_on: micOn,
          cam_on: camOn,
        };
      }
      case RTC_S_ROSTER: {
        const count = this.readU8();
        const participants: RtcParticipant[] = [];
        for (let i = 0; i < count; i += 1) {
          const actor = this.readActor();
          const micOn = this.readU8() !== 0;
          const camOn = this.readU8() !== 0;
          participants.push({ actor, mic_on: micOn, cam_on: camOn });
        }
        return { type: "rtc", kind: "roster", participants };
      }
      case RTC_S_ERROR: {
        const code = this.readString();
        const message = this.readString();
        return { type: "rtc", kind: "error", code, message };
      }
      default:
        throw new Error("Unknown RTC server sub-opcode");
    }
  }

  private readBytes(len: number): Uint8Array {
    this.ensure(len);
    const value = new Uint8Array(this.view.buffer, this.offset, len);
    this.offset += len;
    return value;
  }

  private ensure(len: number) {
    if (this.offset + len > this.view.byteLength) {
      throw new Error("Truncated live chat binary frame");
    }
  }
}
