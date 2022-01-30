use async_graphql::*;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use std::fs::File;
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql_schema::guards::{AdminGuard, Column, Table, UserOwnsGuard};
use crate::types::*;

pub struct Mutation;

impl Mutation {
    async fn delete_helper(result: PgQueryResult) -> Result<bool> {
        if result.rows_affected() == 1 {
            Ok(true)
        } else if result.rows_affected() == 0 {
            Ok(false)
        } else {
            Err(format!("{} rows deleted, not 1", result.rows_affected()).into())
        }
    }
}

#[Object]
impl Mutation {
    #[graphql(guard = "AdminGuard")]
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        administrator: bool,
    ) -> Result<EmoteUser> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        Ok(sqlx::query_as!(
            EmoteUser,
            "INSERT INTO emote_user (username, administrator) VALUES ($1, $2) RETURNING *",
            username,
            administrator
        )
        .fetch_one(&**pool)
        .await?)
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteUser, Column::UUID(uuid)).or(AdminGuard)")]
    async fn delete_user(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let result = sqlx::query!("DELETE FROM emote_user WHERE uuid = ($1)", uuid)
            .execute(&**pool)
            .await?;

        Mutation::delete_helper(result).await
    }

    #[graphql(
        guard = "UserOwnsGuard::new(Table::EmoteUser, Column::UUID(user_uuid)).or(AdminGuard)"
    )]
    async fn create_token(
        &self,
        ctx: &Context<'_>,
        user_uuid: Uuid,
        description: String,
    ) -> Result<String> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        EmoteToken::generate(Arc::clone(&pool), user_uuid, description).await
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteToken, Column::UUID(uuid)).or(AdminGuard)")]
    async fn delete_token(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let result = sqlx::query!("DELETE FROM emote_token WHERE uuid = ($1)", uuid)
            .execute(&**pool)
            .await?;

        Mutation::delete_helper(result).await
    }

    #[graphql(
        guard = "UserOwnsGuard::new(Table::EmoteDir, Column::DirSlug(slug.clone())).or(AdminGuard)"
    )]
    async fn create_dir(
        &self,
        ctx: &Context<'_>,
        slug: String,
        emote_user_uuid: Uuid,
    ) -> Result<EmoteDir> {
        unimplemented!()
    }
    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteDir, Column::UUID(uuid)).or(AdminGuard)")]
    async fn delete_dir(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        // No cascade, so I don't really know if this will work. The only way this will be allowed to work is if only one person owns the directory
        // Step 1: query the number of users in the join table
        // Step 2: if there is only one person left AND that person is you (or the person is privileged), delete all references to the dir from the join table
        // Step 4: delete this dir (will cascade to emotes)
        // Step 5: delete emote files
        let result = sqlx::query!("DELETE FROM emote_dir WHERE uuid = ($1)", uuid)
            .execute(&**pool)
            .await?;

        // TODO delete from the data dir as well
        Mutation::delete_helper(result).await
    }
    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteDir, Column::UUID(dir_uuid)).or(AdminGuard)")]
    async fn add_user_to_dir(
        &self,
        ctx: &Context<'_>,
        user_uuid: Uuid,
        dir_uuid: Uuid,
    ) -> Result<bool> {
        unimplemented!()
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteDir, Column::UUID(dir_uuid)).or(AdminGuard)")]
    async fn upload_emote(
        &self,
        ctx: &Context<'_>,
        dir_uuid: Uuid,
        slug: String,
        emote_file: Upload,
        emote_type: EmoteType,
    ) -> Result<Emote> {
        unimplemented!()
    }

    // It will cascade and delete all emote images
    #[graphql(guard = "UserOwnsGuard::new(Table::Emote, Column::UUID(uuid)).or(AdminGuard)")]
    async fn delete_emote(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let result = sqlx::query!("DELETE FROM emote WHERE uuid = ($1)", uuid)
            .execute(&**pool)
            .await?;

        // TODO delete from the data dir as well
        Mutation::delete_helper(result).await
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::Emote, Column::UUID(emote_uuid)).or(AdminGuard)")]
    async fn create_emote_image(
        &self,
        ctx: &Context<'_>,
        emote_uuid: Uuid,
        width: u64,
        height: u64,
    ) -> Result<EmoteImage> {
        unimplemented!()
    }

    #[graphql(guard = "UserOwnsGuard::new(Table::EmoteImage, Column::UUID(uuid)).or(AdminGuard)")]
    async fn delete_emote_image(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let result = sqlx::query!("DELETE FROM emote_image WHERE uuid = ($1)", uuid)
            .execute(&**pool)
            .await?;

        // TODO delete from the data dir as well
        Mutation::delete_helper(result).await
    }
}
