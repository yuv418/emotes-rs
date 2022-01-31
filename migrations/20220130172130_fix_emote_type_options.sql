-- Add migration script here

ALTER TYPE EMOTE_TYPE RENAME TO EMOTE_TYPE_OLD;
CREATE TYPE EMOTE_TYPE AS ENUM('standard', 'sticker');
ALTER TABLE emote ALTER COLUMN emote_type TYPE EMOTE_TYPE USING emote_type::text::EMOTE_TYPE;
DROP TYPE EMOTE_TYPE_OLD;