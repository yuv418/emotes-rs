use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql_schema::guards::{Column, UserOwnership};

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

// TODO maybe consolidate the boilerplate that's repeated in each function into once function later
#[ComplexObject]
impl EmoteUser {
    async fn tokens(&self, ctx: &Context<'_>) -> Result<Vec<EmoteToken>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteToken,
            "SELECT * FROM emote_token WHERE emote_user_uuid = ($1)",
            self.uuid
        )
        .fetch_all(&**pool)
        .await?)
    }
    async fn dirs(&self, ctx: &Context<'_>) -> Result<Vec<EmoteDir>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteDir,
            "SELECT emote_dir.* FROM emote_dir INNER JOIN emote_user_emote_dir e ON e.emote_dir_uuid = uuid WHERE e.emote_user_uuid = ($1)",
            self.uuid
        ).fetch_all(&**pool).await?)
    }
}
#[async_trait::async_trait]
impl UserOwnership for EmoteUser {
    async fn owned_by(ctx: &Context<'_>, column: &Column, user: &EmoteUser) -> Result<bool> {
        if let Column::UUID(user_uuid) = column {
            return Ok(user_uuid == &user.uuid);
        } else if let Column::Username(username) = column {
            let pool = ctx.data::<Arc<PgPool>>()?; // avoid expensive operation unless necessary

            if let Some(val) = sqlx::query!(
                "SELECT uuid FROM emote_user WHERE username = ($1)",
                username
            )
            .fetch_optional(&**pool)
            .await?
            {
                return Ok(val.uuid == user.uuid);
            }
        }
        Ok(false) // no other value is accepted at the moment
    }
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
#[graphql(complex)]
pub struct EmoteDir {
    pub uuid: Uuid,
    pub slug: String,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
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
    async fn by_slug(&self, pool: Arc<PgPool>, slug: String) -> Result<Self> {
        // 100% of the time, you can split the slug with '/'
        let emote_parts: Vec<&str> = slug.split("/").collect();

        // TODO make a static str and concat the list of columns since we are not being DRY
        Ok(sqlx::query_as!(
            Emote,
            "SELECT emote.uuid, emote.slug, emote_dir_uuid, emote_type as \"emote_type!: EmoteType\", emote.create_time, emote.modify_time FROM emote INNER JOIN emote_dir ON emote.emote_dir_uuid = emote_dir.uuid WHERE emote_dir.slug= ($1) AND emote.slug = ($2)",
            emote_parts[0], emote_parts[1]).fetch_one(&*pool).await?)
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

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteImage {
    pub uuid: Uuid,
    pub width: i32,
    pub height: i32,
    pub emote_uuid: Uuid,
    #[graphql(skip)] // relative to the data dir
    pub image_path: String,
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
#[async_trait::async_trait]
impl UserOwnership for EmoteToken {
    async fn owned_by(ctx: &Context<'_>, column: &Column, user: &EmoteUser) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        if let Column::UUID(token_uuid) = column {
            if let Some(val) = sqlx::query!(
                "SELECT emote_user_uuid FROM emote_token WHERE uuid = ($1)",
                token_uuid
            )
            .fetch_optional(&**pool)
            .await?
            {
                return EmoteUser::owned_by(ctx, &Column::UUID(val.emote_user_uuid), user).await;
            }
        }
        Ok(false)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SerializedEmoteToken {
    token_uuid: u64,
    token: Uuid,
}

impl SerializedEmoteToken {
    pub async fn to_emote_user(
        pool: Arc<PgPool>,
        serialized_token: &str,
    ) -> Result<Option<EmoteUser>> {
        // TODO Error message should be "Invalid token value; failed to parse token."
        let deserialized_token: SerializedEmoteToken = serde_json::from_slice(
            &base64::decode(serialized_token).expect("Failed to decode serialized token"),
        )?;

        if let Some(emote_token) = sqlx::query_as!(
            EmoteToken,
            "SELECT * FROM emote_token WHERE uuid=($1)",
            deserialized_token.token
        )
        .fetch_optional(&*pool)
        .await?
        {
            // Found a token, does it correspond to a user?
            // TODO handle when there is no user but a token
            // Shouldn't we delete or warn?
            Ok(sqlx::query_as!(
                EmoteUser,
                "SELECT * FROM emote_user WHERE uuid=($1)",
                emote_token.emote_user_uuid
            )
            .fetch_optional(&*pool)
            .await?)
        } else {
            // No token found
            Ok(None)
        }
    }
}
