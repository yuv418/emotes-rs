-- Add migration script here


ALTER TABLE emote_token DROP CONSTRAINT emote_token_emote_user_uuid_fkey;
ALTER TABLE emote_token ADD CONSTRAINT emote_token_emote_user_uuid_fkey FOREIGN KEY (emote_user_uuid) REFERENCES emote_user(uuid) ON DELETE CASCADE;

ALTER TABLE emote DROP CONSTRAINT emote_emote_dir_uuid_fkey;
ALTER TABLE emote ADD CONSTRAINT emote_emote_dir_uuid_fkey FOREIGN KEY (emote_dir_uuid) REFERENCES emote_dir(uuid) ON DELETE CASCADE;

ALTER TABLE emote_image DROP CONSTRAINT emote_image_emote_uuid_fkey;
ALTER TABLE emote_image ADD CONSTRAINT emote_image_emote_uuid_fkey FOREIGN KEY (emote_uuid) REFERENCES emote(uuid) ON DELETE CASCADE;
