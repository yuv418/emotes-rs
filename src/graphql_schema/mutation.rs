use async_graphql::*;
use std::fs::File;
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql_schema::guards::AdminGuard;
use crate::types::*;

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard = "AdminGuard")]
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        administrator: bool,
    ) -> Result<EmoteUser> {
        unimplemented!()
    }
    #[graphql(guard = "AdminGuard")]
    async fn delete_user(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn create_token(
        &self,
        ctx: &Context<'_>,
        user_uuid: Uuid,
        description: String,
    ) -> Result<EmoteToken> {
        unimplemented!()
    }

    async fn delete_token(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn create_dir(
        &self,
        ctx: &Context<'_>,
        slug: String,
        emote_user_uuid: Uuid,
    ) -> Result<EmoteDir> {
        unimplemented!()
    }
    async fn delete_dir(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn upload_emote(
        &self,
        ctx: &Context<'_>,
        slug: String,
        emote_file: Upload,
        emote_type: EmoteType,
    ) -> Result<Emote> {
        unimplemented!()
    }

    // It will cascade and delete all emote images
    async fn delete_emote(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn create_emote_image(
        &self,
        ctx: &Context<'_>,
        emote_uuid: Uuid,
        width: u64,
        height: u64,
    ) -> Result<EmoteImage> {
        unimplemented!()
    }

    async fn delete_emote_image(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        unimplemented!()
    }
}
