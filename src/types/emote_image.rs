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
pub struct EmoteImage {
    pub uuid: Uuid,
    pub width: i32,
    pub height: i32,
    pub emote_uuid: Uuid,
    #[graphql(skip)] // relative to the data dir
    pub image_path: String,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
    // First the image gets inserted, then it is resized, then this is updated when the image is saved to the data dir
    // We need this since the image is saved as <uuid>.<extension>, and we don't get the UUID until we actually insert.
    pub processing: bool,
}
