use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::*;

use crate::graphql_schema::guards::{Column, UserOwnership};
#[derive(sqlx::Type, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Enum)]
#[sqlx(type_name = "emote_type", rename_all = "lowercase")]
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
    async fn images(&self, ctx: &Context<'_>) -> Result<Vec<EmoteImage>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteImage,
            "SELECT * FROM emote_image WHERE emote_uuid = ($1)",
            self.uuid
        )
        .fetch_all(&**pool)
        .await?)
    }
}

impl Emote {
    // Don't really see a way to keep this DRY
    pub async fn all(pool: Arc<PgPool>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Emote,
            "SELECT emote.uuid, emote.slug, emote_dir_uuid, emote_type as \"emote_type!: EmoteType\", emote.create_time, emote.modify_time FROM emote")
            .fetch_all(&*pool).await?)
    }
    pub async fn by_uuid(pool: Arc<PgPool>, uuid: Uuid) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Emote,
            "SELECT emote.uuid, emote.slug, emote_dir_uuid, emote_type as \"emote_type!: EmoteType\", emote.create_time, emote.modify_time FROM emote WHERE emote.uuid = ($1)",
            uuid).fetch_optional(&*pool).await?)
    }
    pub async fn by_slug(pool: Arc<PgPool>, slug: String) -> Result<Option<Self>> {
        // 100% of the time, you can split the slug with '/'
        let emote_parts: Vec<&str> = slug.split("/").collect();

        // TODO make a static str and concat the list of columns since we are not being DRY
        // might not be possible, though
        Ok(sqlx::query_as!(
            Emote,
            "SELECT emote.uuid, emote.slug, emote_dir_uuid, emote_type as \"emote_type!: EmoteType\", emote.create_time, emote.modify_time FROM emote INNER JOIN emote_dir ON emote.emote_dir_uuid = emote_dir.uuid WHERE emote_dir.slug= ($1) AND emote.slug = ($2)",
            emote_parts[0], emote_parts[1]).fetch_optional(&*pool).await?)
    }
}

#[async_trait::async_trait]
impl UserOwnership for Emote {
    async fn owned_by(ctx: &Context<'_>, column: &Column, user: &EmoteUser) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let dir_col = if let Column::UUID(emote_uuid) = column {
            if let Some(val) = sqlx::query!(
                "SELECT emote_dir_uuid FROM emote WHERE uuid = ($1)",
                emote_uuid
            )
            .fetch_optional(&**pool)
            .await?
            {
                Column::UUID(val.emote_dir_uuid)
            } else {
                return Ok(false);
            }
        } else if let Column::EmoteSlug(emote_slug) = column {
            let emote_parts: Vec<&str> = emote_slug.split("/").collect();
            Column::DirSlug(emote_parts[0].to_owned())
        } else {
            return Ok(false);
        };

        EmoteDir::owned_by(ctx, &dir_col, user).await
    }
}
#[async_trait::async_trait]
impl UserOwnership for EmoteImage {
    async fn owned_by(ctx: &Context<'_>, column: &Column, user: &EmoteUser) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        if let Column::UUID(emote_image_uuid) = column {
            if let Some(val) = sqlx::query!(
                "SELECT emote_uuid FROM emote_image WHERE uuid = ($1)",
                emote_image_uuid
            )
            .fetch_optional(&**pool)
            .await?
            {
                return Emote::owned_by(ctx, &Column::UUID(val.emote_uuid), user).await;
            }
        }

        Ok(false)
    }
}
