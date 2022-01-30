-- Add migration script here

ALTER TABLE emote_image ADD COLUMN processing BOOLEAN DEFAULT false;
