use async_graphql::*;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql_schema::guards::{AdminGuard, Column, Table, UserOwnsGuard};
use crate::types::*;

pub struct Query;

#[Object]
impl Query {
    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteUser, Column::UUID(uuid)).or(AdminGuard)")]
    pub async fn user(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Option<EmoteUser>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteUser,
            "SELECT * FROM emote_user WHERE uuid = ($1)",
            uuid
        )
        .fetch_optional(&**pool)
        .await?)
    }
    #[graphql(
        guard = "UserOwnsGuard::new(Table::EmoteUser, Column::Username(username.clone())).or(AdminGuard)"
    )]
    async fn user_by_username(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> Result<Option<EmoteUser>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteUser,
            "SELECT * FROM emote_user WHERE username = ($1)",
            username
        )
        .fetch_optional(&**pool)
        .await?)
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_users(&self, ctx: &Context<'_>) -> Result<Vec<EmoteUser>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(EmoteUser, "SELECT * FROM emote_user",)
            .fetch_all(&**pool)
            .await?)
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteToken, Column::UUID(uuid)).or(AdminGuard)")]
    async fn token(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Option<EmoteToken>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteToken,
            "SELECT * FROM emote_token WHERE uuid = ($1)",
            uuid
        )
        .fetch_optional(&**pool)
        .await?)
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_tokens(&self, ctx: &Context<'_>) -> Result<Vec<EmoteToken>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(EmoteToken, "SELECT * FROM emote_token")
            .fetch_all(&**pool)
            .await?)
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteDir, Column::UUID(uuid)).or(AdminGuard)")]
    async fn dir(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Option<EmoteDir>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(
            sqlx::query_as!(EmoteDir, "SELECT * FROM emote_dir WHERE uuid = ($1)", uuid)
                .fetch_optional(&**pool)
                .await?,
        )
    }
    // no, you want to do fields
    #[graphql(
        guard = "UserOwnsGuard::new(Table::EmoteDir, Column::DirSlug(slug.clone())).or(AdminGuard)"
    )]
    async fn dir_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<Option<EmoteDir>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(
            sqlx::query_as!(EmoteDir, "SELECT * FROM emote_dir WHERE slug = ($1)", slug)
                .fetch_optional(&**pool)
                .await?,
        )
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_dirs(&self, ctx: &Context<'_>) -> Result<Vec<EmoteDir>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(EmoteDir, "SELECT * FROM emote_dir")
            .fetch_all(&**pool)
            .await?)
    }
    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteDir, Column::UUID(dir_uuid)).or(AdminGuard)")]
    async fn user_privileged_for_dir(
        &self,
        ctx: &Context<'_>,
        user_uuid: Uuid,
        dir_uuid: Uuid,
    ) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        EmoteDir::user_privileged_for_dir(Arc::clone(&pool), user_uuid, dir_uuid).await
    }

    // TODO add verbs for directory privileges

    #[graphql(guard = "UserOwnsGuard::new(Table::Emote, Column::UUID(uuid)).or(AdminGuard)")]
    async fn emote(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Option<Emote>> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        Emote::by_uuid(Arc::clone(&pool), uuid).await
    }
    #[graphql(
        guard = "UserOwnsGuard::new(Table::Emote, Column::EmoteSlug(slug.clone())).or(AdminGuard)"
    )]
    async fn emote_by_slug(&self, ctx: &Context<'_>, slug: String) -> Result<Option<Emote>> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        Emote::by_slug(Arc::clone(&pool), slug).await
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_emotes(&self, ctx: &Context<'_>) -> Result<Vec<Emote>> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        Emote::all(Arc::clone(&pool)).await
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteImage, Column::UUID(uuid)).or(AdminGuard)")]
    async fn emote_image(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<Option<EmoteImage>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteImage,
            "SELECT * FROM emote_image WHERE uuid = ($1)",
            uuid
        )
        .fetch_optional(&**pool)
        .await?)
    }
    #[graphql(guard = "AdminGuard")]
    async fn all_emote_images(&self, ctx: &Context<'_>) -> Result<Vec<EmoteImage>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(EmoteImage, "SELECT * FROM emote_image",)
            .fetch_all(&**pool)
            .await?)
    }
}
