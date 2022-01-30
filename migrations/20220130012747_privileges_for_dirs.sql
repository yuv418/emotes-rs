-- Add migration script here

ALTER TABLE emote_user_emote_dir ADD COLUMN privileged BOOLEAN DEFAULT false;
