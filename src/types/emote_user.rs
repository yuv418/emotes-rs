use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::graphql_schema::guards::{Column, UserOwnership};
use crate::types::*;

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
