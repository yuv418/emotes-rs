
use sqlx::postgres::PgQueryResult;
use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;


use crate::graphql_schema::guards::{Column, UserOwnership};
use crate::types::*;

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
#[graphql(complex)]
pub struct EmoteDir {
    pub uuid: Uuid,
    pub slug: String,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}

impl EmoteDir {
    pub async fn delete(pool: Arc<PgPool>, uuid: Uuid) -> Result<PgQueryResult> {
        // cascade was pointless
        for emote_uuid in
            sqlx::query!("SELECT uuid FROM emote WHERE emote_dir_uuid = ($1)", uuid)
                .fetch_all(&*pool)
                .await?
        {
            EmoteImage::delete(Arc::clone(&pool), emote_uuid.uuid).await?;
        }

        Ok(sqlx::query!("DELETE FROM emote_dir WHERE uuid = ($1)", uuid)
            .execute(&*pool)
            .await?)
    }
    pub async fn user_privileged_for_dir(
        pool: Arc<PgPool>,
        user_uuid: Uuid,
        dir_uuid: Uuid,
    ) -> Result<bool> {
        if let Some(val) = 
            sqlx::query!("SELECT privileged FROM emote_user_emote_dir WHERE emote_user_uuid = ($1) AND emote_dir_uuid = ($2)", user_uuid, dir_uuid)
                .fetch_optional(&*pool)
                .await? {
                    return Ok(val.privileged);
                }

        Ok(false)
    }
}

// TODO add regular impl to check if user (by uuid) is privileged for this dir
#[ComplexObject]
impl EmoteDir {
    // Dealing with many-to-many relationship here
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<EmoteUser>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteUser,
            "SELECT emote_user.* FROM emote_user INNER JOIN emote_user_emote_dir e ON e.emote_user_uuid = uuid WHERE e.emote_dir_uuid = ($1)",
            self.uuid
        ).fetch_all(&**pool).await?)
    }
    async fn emotes(&self, ctx: &Context<'_>) -> Result<Vec<Emote>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        // You have to do this when querying an enum
        Ok(sqlx::query_as!(
            Emote,
            "SELECT uuid, slug, emote_dir_uuid, emote_type as \"emote_type!: EmoteType\", create_time, modify_time FROM emote WHERE emote_dir_uuid = ($1)",
            self.uuid
        )
        .fetch_all(&**pool)
        .await?)
    }
}

#[async_trait::async_trait]
impl UserOwnership for EmoteDir {
    async fn owned_by(ctx: &Context<'_>, column: &Column, user: &EmoteUser) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let dir_uuid = if let Column::UUID(dir_uuid) = column {
            *dir_uuid
        } else if let Column::DirSlug(dir_slug) = column {
            // TODO replace with function from query.rs
            if let Some(val) = sqlx::query!("SELECT uuid FROM emote_dir WHERE slug=($1)", dir_slug)
                .fetch_optional(&**pool)
                .await?
            {
                val.uuid
            } else {
                return Ok(false); // no uuid for slug, does not exist
            }
        } else {
            return Ok(false); // no usernames
        };

        if let None = sqlx::query!(
            "SELECT emote_user.uuid FROM emote_user INNER JOIN emote_user_emote_dir e ON e.emote_user_uuid = uuid WHERE e.emote_dir_uuid = ($1) AND emote_user.uuid = ($2)",
            dir_uuid,
            user.uuid
        ).fetch_optional(&**pool).await? {
            Ok(false)
        }
        else {
            Ok(true)
        }
    }
}
