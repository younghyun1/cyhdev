ALTER TABLE comments
    ADD CONSTRAINT comments_comment_content_character_length
    CHECK (char_length(comment_content) BETWEEN 1 AND 4000)
    NOT VALID;

COMMENT ON CONSTRAINT comments_comment_content_character_length ON comments IS
    'Enforces the public blog comment boundary for new and updated rows; legacy rows remain readable until separately remediated.';
