-- Recent-history and older-page reads filter one room, exclude moderation
-- tombstones, and order by (created, id) descending with a row-comparison
-- cursor. This partial index serves that order and bounds the cursor scan; it
-- supersedes the room/created index, which no other query uses.
CREATE INDEX live_chat_messages_room_visible_keyset_idx
    ON live_chat_messages (room_key, message_created_at DESC, live_chat_message_id DESC)
    WHERE message_deleted_at IS NULL;

DROP INDEX live_chat_messages_room_created_idx;
