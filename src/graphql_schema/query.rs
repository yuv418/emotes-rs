use async_graphql::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql_schema::guards::AdminGuard;
use crate::types::*;

pub struct Query;

#[Object]
impl Query {
    // TODO ADMIN ONLY
    #[graphql(guard = "AdminGuard")]
    async fn users(&self, ctx: &Context<'_>, uuid: Option<Uuid>) -> Result<Vec<EmoteUser>> {
        Ok(vec![])
    }

    async fn token(&self, ctx: &Context<'_>, uuid: Option<Uuid>) -> Result<EmoteToken> {
        unimplemented!()
    }

    async fn dir(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<EmoteDir> {
        unimplemented!()
    }
    // no, you want to do fields
    async fn dir_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<EmoteDir> {
        unimplemented!()
    }
    // Admin only
    async fn all_dirs(&self, ctx: &Context<'_>) -> Result<Vec<EmoteDir>> {
        Ok(vec![])
    }

    async fn emote(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Emote> {
        unimplemented!()
    }
    async fn emote_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<Vec<Emote>> {
        Ok(vec![])
    }
    // Admin only
    async fn all_emotes(&self, ctx: &Context<'_>) -> Result<Vec<Emote>> {
        Ok(vec![])
    }

    async fn emote_image(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<EmoteImage> {
        unimplemented!()
    }
    async fn emote_image_by_size(
        &self,
        ctx: &Context<'_>,
        emote_uuid: Uuid,
        width: u64,
        height: u64,
    ) -> Result<EmoteImage> {
        unimplemented!()
    }
    // Admin only
    async fn all_emote_images(&self, ctx: &Context<'_>) -> Result<Vec<EmoteImage>> {
        Ok(vec![])
    }
}
