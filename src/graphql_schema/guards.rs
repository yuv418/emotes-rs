use async_graphql::{Context, Guard, Result};

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

        Err("You are not an administrator, so you do not have access to this resource".into())
    }
}
