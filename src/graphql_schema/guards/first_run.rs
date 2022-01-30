use async_graphql::{Context, Guard, Result};
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;

use crate::types::*;
use lazy_static::lazy_static;

pub struct FirstRunGuard;

lazy_static! {
    pub static ref FIRST_RUN: RwLock<bool> = RwLock::new(false);
}
#[async_trait::async_trait]
impl Guard for FirstRunGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if *FIRST_RUN.read().unwrap() {
            Ok(())
        } else {
            Err("First run mode is disabled.".into())
        }
    }
}
