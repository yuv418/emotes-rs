use async_graphql::*;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::graphql_schema::guards::{
    AdminGuard, Column, Table, UserDirPrivilegedGuard, UserOwnsGuard,
};
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
        let pool = ctx.data::<Arc<PgPool>>()?;
        let emote_dir: EmoteDir = sqlx::query_as!(
            EmoteDir,
            "INSERT INTO emote_dir (slug) VALUES ($1) RETURNING *",
            slug
        )
        .fetch_one(&**pool)
        .await?;

        // insert relation for user. Privileged since creator.
        let result = sqlx::query!("INSERT INTO emote_user_emote_dir (emote_user_uuid, emote_dir_uuid, privileged) VALUES ($1, $2, $3)", emote_user_uuid, emote_dir.uuid, true).execute(&**pool).await?;
        if let Ok(true) = Mutation::delete_helper(result).await {
            Ok(emote_dir)
        } else {
            // TODO rollback
            Err("failed to insert emote_dir".into())
        }
    }
    #[graphql(guard = "UserDirPrivilegedGuard::new(uuid).or(AdminGuard)")]
    async fn delete_dir(&self, ctx: &Context<'_>, uuid: Uuid) -> Result<bool> {
        let user = ctx.data::<EmoteUser>()?;
        let pool = ctx.data::<Arc<PgPool>>()?;
        // Step 1: query the number of users in the join table
        // Step 2: if there is only one person left AND that person is you, delete all references to the dir from the join table
        // Step 4: delete this dir (will cascade to emotes)
        // Step 5: delete emote files
        let dir_owners = sqlx::query!(
            "SELECT emote_user_uuid FROM emote_user_emote_dir WHERE emote_dir_uuid = ($1)",
            uuid,
        )
        .fetch_all(&**pool)
        .await?;
        if dir_owners.len() == 1 {
            // one owner
            if user.administrator || dir_owners[0].emote_user_uuid == user.uuid {
                // one person left, that person is you. We don't have to check privilege since it's alread done for us

                let result = sqlx::query!("DELETE FROM emote_dir WHERE uuid = ($1)", uuid)
                    .execute(&**pool)
                    .await?;

                // TODO all emote data/dirs MUST be deleted here

                Mutation::delete_helper(result).await
            } else {
                Err("You are not authorized to delete this resource as you are not an administrator or the single owner of this dir.".into())
            }
        } else {
            Err("You can only delete a dir when it has one owner remaining.".into())
        }

        // TODO delete from the data dir as well
    }

    #[graphql(guard = "UserDirPrivilegedGuard::new(dir_uuid).or(AdminGuard)")]
    async fn add_user_to_dir(
        &self,
        ctx: &Context<'_>,
        user_uuid: Uuid,
        dir_uuid: Uuid,
        privileged: bool,
    ) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let result = sqlx::query!("INSERT INTO emote_user_emote_dir (emote_user_uuid, emote_dir_uuid, privileged) VALUES ($1, $2, $3)",
                                  user_uuid,
                                  dir_uuid,
                                  privileged).execute(&**pool).await?;

        Mutation::delete_helper(result).await // we can use the delete helper since it just checks how many rows were changed. TODO change the delete helper's name
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
    // TODO do we want to let anyone do this, or only privileged dir owners?
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
