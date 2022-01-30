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

use crate::types::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::graphql_schema::guards::{Column, UserOwnership};
#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteToken {
    pub uuid: Uuid,
    pub emote_user_uuid: Uuid,
    pub description: String,
    #[graphql(skip)]
    pub token_hash: String,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}

impl EmoteToken {
    // Generates a fully serialized emote token as a String for a given user UUID
    pub async fn generate(
        pool: Arc<PgPool>,
        user_uuid: Uuid,
        description: String,
    ) -> async_graphql::Result<String> {
        // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters

        let gen_pw: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut thread_rng());
        let gen_pw_hash = if let Ok(hash) = argon2.hash_password(gen_pw.as_bytes(), &salt) {
            hash
        } else {
            return Err("Failed to hash token".into());
        };

        if let Some(val) = sqlx::query!("INSERT INTO emote_token (emote_user_uuid, description, token_hash) VALUES ($1, $2, $3) RETURNING uuid",
                     user_uuid, description, gen_pw_hash.to_string()
        ).fetch_optional(&*pool).await? {
            let serialized_token = SerializedEmoteToken {
                token_uuid: val.uuid,
                token: gen_pw.to_string()
            };

            Ok(base64::encode_config(serde_json::to_string(&serialized_token)?, base64::URL_SAFE))

        } else {
            Err("Failed to insert emote_token".into())
        }
    }
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
    token_uuid: Uuid,
    token: String,
}

impl SerializedEmoteToken {
    pub async fn to_emote_user(
        pool: Arc<PgPool>,
        serialized_token: &str,
    ) -> Result<Option<EmoteUser>> {
        // TODO Error message should be "Invalid token value; failed to parse token."
        let deserialized_token: SerializedEmoteToken = serde_json::from_slice(
            &base64::decode_config(serialized_token, base64::URL_SAFE)
                .expect("Failed to decode serialized token"),
        )?;

        if let Some(emote_token) = sqlx::query_as!(
            EmoteToken,
            "SELECT * FROM emote_token WHERE uuid=($1)",
            deserialized_token.token_uuid
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
