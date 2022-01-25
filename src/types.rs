use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// TODO add relations for SeaORM and remove
// Vecs
// TODO add create/update time fields

#[derive(Serialize, Deserialize, Debug, DeriveEntityModel)]
pub struct EmoteUser {
    uuid: Uuid,
    dirs: Vec<EmoteDir>,
    tokens: Vec<EmoteToken>,
    username: String,
    administrator: bool,
}

#[derive(Serialize, Deserialize, Debug, DeriveEntityModel)]
pub struct EmoteDir {
    uuid: Uuid,
    slug: String,
    emotes: Vec<Emote>,
    // Make sure that an emote slug and a
    // child_dir slug do not collid when we insert!
    child_dirs: Vec<EmoteDir>,
}

#[derive(Serialize, Deserialize, Debug, DeriveEntityModel)]
pub enum EmoteType {
    Animated,
    Still,
    Sticker,
}

#[derive(Serialize, Deserialize, Debug, DeriveEntityModel)]
pub struct Emote {
    uuid: Uuid,
    slug: String,
    emote_type: EmoteType,
    // Will be empty be default, we will create a
    // load_images() function to read this from the
    // database tree
    images: Vec<EmoteImage>,
}

#[derive(Serialize, Deserialize, Debug, DeriveEntityModel)]
pub struct EmoteImage {
    uuid: Uuid,
    width: u64,
    height: u64,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, DeriveEntityModel)]
pub struct EmoteToken {
    uuid: Uuid,
    token_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::emote::Entity")]
    Emote,
    EmoteImage,
    EmoteType,
    EmoteDir,
    EmoteToken,
}
