use async_graphql::{Context, Guard, Result};
use lazy_static::lazy_static;
use std::sync::RwLock;

pub struct FirstRunGuard;

lazy_static! {
    pub static ref FIRST_RUN: RwLock<bool> = RwLock::new(false);
}
#[async_trait::async_trait]
impl Guard for FirstRunGuard {
    async fn check(&self, _ctx: &Context<'_>) -> Result<()> {
        if *FIRST_RUN.read().unwrap() {
            Ok(())
        } else {
            Err("First run mode is disabled.".into())
        }
    }
}
