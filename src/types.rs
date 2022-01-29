use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// TODO add create/update time fields

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
#[graphql(complex)]
pub struct EmoteUser {
    pub uuid: Uuid,
    pub username: String,
    pub administrator: bool,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}

#[ComplexObject]
impl EmoteUser {
    async fn tokens(&self, ctx: &Context<'_>) -> Vec<EmoteImage> {
        unimplemented!()
    }
    async fn dirs(&self, ctx: &Context<'_>) -> Vec<EmoteImage> {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
#[graphql(complex)]
pub struct EmoteDir {
    pub uuid: Uuid,
    pub slug: String,
    pub emote_user_uuid: Uuid,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}
#[ComplexObject]
impl EmoteDir {
    async fn emotes(&self, ctx: &Context<'_>) -> Vec<EmoteImage> {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Enum)]
pub enum EmoteType {
    Animated,
    Still,
    Sticker,
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Emote {
    pub uuid: Uuid,
    pub slug: String,
    pub emote_dir_uuid: Uuid,
    pub emote_type: EmoteType,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}

#[ComplexObject]
impl Emote {
    async fn images(&self, ctx: &Context<'_>) -> Vec<EmoteImage> {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteImage {
    pub uuid: Uuid,
    pub width: u64,
    pub height: u64,
    pub emote_uuid: Uuid,
    #[graphql(skip)]
    pub path: String,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteToken {
    pub uuid: Uuid,
    pub emote_user_uuid: Uuid,
    pub description: String,
    pub token_hash: String,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}
