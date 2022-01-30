-- Add migration script here

ALTER TABLE emote_image ALTER COLUMN processing SET NOT NULL;
ALTER TABLE emote_user_emote_dir ALTER COLUMN privileged SET NOT NULL;
