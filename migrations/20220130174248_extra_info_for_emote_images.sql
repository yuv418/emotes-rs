-- Add migration script here

ALTER TABLE emote_image ADD COLUMN original BOOLEAN NOT NULL;
ALTER TABLE emote_image ADD COLUMN content_type TEXT NOT NULL;
