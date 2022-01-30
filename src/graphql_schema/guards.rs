use async_graphql::{Context, Guard, Result};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::*;

pub struct AdminGuard;

#[async_trait::async_trait]
impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(emote_user) = ctx.data_opt::<EmoteUser>() {
            if emote_user.administrator {
                return Ok(());
            }
        }

        // With UserOwnsGuard, we might have to rework this message a bit
        Err("You are not an administrator, so you do not have access to this resource".into())
    }
}
pub struct UserDirPrivilegedGuard {
    dir_uuid: Uuid,
}

impl UserDirPrivilegedGuard {
    pub fn new(dir_uuid: Uuid) -> Self {
        Self { dir_uuid }
    }
}

#[async_trait::async_trait]
impl Guard for UserDirPrivilegedGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(emote_user) = ctx.data_opt::<EmoteUser>() {
            let pool = ctx.data::<Arc<PgPool>>()?;
            if EmoteDir::user_privileged_for_dir(Arc::clone(&pool), emote_user.uuid, self.dir_uuid)
                .await?
            {
                return Ok(());
            }
        }
        Err("You are not a privileged member of this directory, so you cannot access this resource.".into())
    }
}

// User ownership guards

#[async_trait::async_trait]
pub trait UserOwnership {
    async fn owned_by(ctx: &Context<'_>, column: &Column, user: &EmoteUser) -> Result<bool>;
}

#[derive(PartialEq)]
pub enum Column {
    DirSlug(String),
    EmoteSlug(String),
    Username(String),
    UUID(Uuid),
}

pub enum Table {
    EmoteUser,
    EmoteDir,
    Emote,
    EmoteImage,
    EmoteToken,
}

// We create a match for the table to a struct,
// we manually map the username/slug (those are the two non-uuid allowable column values) to the uuid
// Then we run it through the ownership trait to see if the user owns the resource.
pub struct UserOwnsGuard {
    table: Table,
    column: Column,
}

impl UserOwnsGuard {
    pub fn new(table: Table, column: Column) -> Self {
        UserOwnsGuard { table, column }
    }
}

#[async_trait::async_trait]
impl Guard for UserOwnsGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(emote_user) = ctx.data_opt::<EmoteUser>() {
            if match &self.table {
                Table::EmoteUser => EmoteUser::owned_by(ctx, &self.column, &emote_user).await?,
                Table::EmoteDir => EmoteDir::owned_by(ctx, &self.column, &emote_user).await?,
                Table::Emote => Emote::owned_by(ctx, &self.column, &emote_user).await?,
                Table::EmoteImage => EmoteImage::owned_by(ctx, &self.column, &emote_user).await?,
                Table::EmoteToken => EmoteToken::owned_by(ctx, &self.column, &emote_user).await?,
            } {
                return Ok(());
            }
        }
        Err("You don't own this resource; unauthorized".into())
    }
}
