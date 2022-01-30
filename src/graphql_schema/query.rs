use async_graphql::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql_schema::guards::{AdminGuard, Column, Table, UserOwnsGuard};
use crate::types::*;

pub struct Query;

#[Object]
impl Query {
    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteUser, Column::UUID(uuid)).or(AdminGuard)")]
    async fn user_by_uuid(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Vec<EmoteUser>> {
        Ok(vec![])
    }
    #[graphql(
        guard = "UserOwnsGuard::new(Table::EmoteUser, Column::Username(username.clone())).or(AdminGuard)"
    )]
    async fn user_by_username(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> Result<Vec<EmoteUser>> {
        Ok(vec![])
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_users(&self, ctx: &Context<'_>) -> Result<Vec<EmoteUser>> {
        Ok(vec![])
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteToken, Column::UUID(uuid)).or(AdminGuard)")]
    async fn token(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<EmoteToken> {
        unimplemented!()
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_tokens(&self, ctx: &Context<'_>) -> Result<EmoteToken> {
        unimplemented!()
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteDir, Column::UUID(uuid)).or(AdminGuard)")]
    async fn dir(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<EmoteDir> {
        unimplemented!()
    }
    // no, you want to do fields
    #[graphql(
        guard = "UserOwnsGuard::new(Table::EmoteDir, Column::DirSlug(slug.clone())).or(AdminGuard)"
    )]
    async fn dir_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<EmoteDir> {
        unimplemented!()
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_dirs(&self, ctx: &Context<'_>) -> Result<Vec<EmoteDir>> {
        Ok(vec![])
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::Emote, Column::UUID(uuid)).or(AdminGuard)")]
    async fn emote(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Emote> {
        unimplemented!()
    }
    #[graphql(
        guard = "UserOwnsGuard::new(Table::Emote, Column::EmoteSlug(slug.clone())).or(AdminGuard)"
    )]
    async fn emote_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<Vec<Emote>> {
        Ok(vec![])
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_emotes(&self, ctx: &Context<'_>) -> Result<Vec<Emote>> {
        Ok(vec![])
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteImage, Column::UUID(uuid)).or(AdminGuard)")]
    async fn emote_image(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<EmoteImage> {
        unimplemented!()
    }
    #[graphql(guard = "UserOwnsGuard::new(Table::Emote, Column::UUID(emote_uuid)).or(AdminGuard)")]
    async fn emote_image_by_size(
        &self,
        ctx: &Context<'_>,
        emote_uuid: Uuid,
        width: u64,
        height: u64,
    ) -> Result<EmoteImage> {
        unimplemented!()
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_emote_images(&self, ctx: &Context<'_>) -> Result<Vec<EmoteImage>> {
        Ok(vec![])
    }
}
