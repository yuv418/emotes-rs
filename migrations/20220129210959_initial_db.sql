-- Add migration script here

CREATE TABLE IF NOT EXISTS emote_user (
       uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       username VARCHAR(200) UNIQUE NOT NULL,
       administrator BOOLEAN NOT NULL,
       create_time TIMESTAMP WITH TIME ZONE NOT NULL,
       modify_time TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS emote_token (
       uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       emote_user_uuid UUID REFERENCES emote_user (uuid) NOT NULL,
       description VARCHAR(500) UNIQUE NOT NULL,
       token_hash TEXT NOT NULL,
       create_time TIMESTAMP WITH TIME ZONE NOT NULL,
       modify_time TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS emote_dir (
       uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       slug VARCHAR(200) UNIQUE NOT NULL,
       create_time TIMESTAMP WITH TIME ZONE NOT NULL,
       modify_time TIMESTAMP WITH TIME ZONE
);

-- don't bother with the create/update time for a join table
CREATE TABLE IF NOT EXISTS emote_user_emote_dir (
       emote_user_uuid UUID REFERENCES emote_user (uuid) NOT NULL,
       emote_dir_uuid  UUID REFERENCES emote_dir  (uuid) NOT NULL,
       CONSTRAINT unique_user_dir UNIQUE (emote_user_uuid, emote_dir_uuid)
);

CREATE TYPE EMOTE_TYPE AS ENUM ('animated', 'still', 'sticker');

CREATE TABLE IF NOT EXISTS emote (
       uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       slug VARCHAR(200) UNIQUE NOT NULL,
       emote_dir_uuid UUID REFERENCES emote_dir (uuid) NOT NULL,
       emote_type EMOTE_TYPE NOT NULL,
       create_time TIMESTAMP WITH TIME ZONE NOT NULL,
       modify_time TIMESTAMP WITH TIME ZONE,
       CONSTRAINT unique_slug_per_dir UNIQUE (slug, emote_dir_uuid)
);

CREATE TABLE IF NOT EXISTS emote_image(
       uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       width INT NOT NULL,
       height INT NOT NULL,
       emote_uuid UUID REFERENCES emote (uuid) NOT NULL,
       image_path TEXT UNIQUE NOT NULL,
       create_time TIMESTAMP WITH TIME ZONE NOT NULL,
       modify_time TIMESTAMP WITH TIME ZONE,
       CONSTRAINT unique_size_per_emote UNIQUE (width, height, emote_uuid)
);
