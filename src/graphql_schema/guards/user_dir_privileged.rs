use async_graphql::{Context, Guard, Result};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::*;

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
