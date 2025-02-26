BEGIN;

CREATE TABLE IF NOT EXISTS rust_simple_chat.messages
(
    message_id      bigserial,
    message_content varchar(300),
    user_id         integer,
    posted_at       timestamptz
);

COMMIT;
