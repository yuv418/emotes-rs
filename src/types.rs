use async_graphql::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// TODO add create/update time fields

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteUser {
    uuid: Uuid,
    username: String,
    administrator: bool,
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteDir {
    uuid: Uuid,
    slug: String,
    emote_user_uuid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Enum)]
pub enum EmoteType {
    Animated,
    Still,
    Sticker,
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct Emote {
    uuid: Uuid,
    slug: String,
    emote_dir_uuid: Uuid,
    emote_type: EmoteType,
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteImage {
    uuid: Uuid,
    width: u64,
    height: u64,
    emote_uuid: Uuid,
    path: String,
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteToken {
    uuid: Uuid,
    token_hash: String,
}
