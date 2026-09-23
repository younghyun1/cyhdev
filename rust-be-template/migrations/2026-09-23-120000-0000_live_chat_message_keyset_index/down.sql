-- Index-only change: rollback restores the previous access path and removes no data.
CREATE INDEX live_chat_messages_room_created_idx
    ON live_chat_messages (room_key, message_created_at DESC);

DROP INDEX live_chat_messages_room_visible_keyset_idx;
