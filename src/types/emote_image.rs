use async_graphql::*;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use libvips::{ops, VipsApp, VipsImage};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{
    fs::{write, File},
    io::Read,
};
use uuid::Uuid;

use crate::{config::EMOTES_CONFIG, image::ImageProcessor};

lazy_static! {
    static ref VIPS: VipsApp = {
        let app = VipsApp::new("Emotes Vips Resizer", false).expect("failed to run vips");
        app.concurrency_set(20);
        app
    };
}

#[derive(Serialize, Deserialize, Debug, SimpleObject)]
pub struct EmoteImage {
    pub uuid: Uuid,
    pub width: i32,
    pub height: i32,
    pub emote_uuid: Uuid,
    // There used to be an emote_path here, but that can automatically be computed based on the content type
    // Original uploaded image
    pub original: bool,
    pub content_type: String,
    // First the image gets inserted, then it is resized, then this is updated when the image is saved to the data dir
    // We need this since the image is saved as <uuid>.<extension>, and we don't get the UUID until we actually insert.
    pub processing: bool,
    pub create_time: DateTime<Utc>,
    pub modify_time: Option<DateTime<Utc>>,
}

impl EmoteImage {
    pub fn get_emote_bytes(&self) -> anyhow::Result<Vec<u8>> {
        use crate::storage::{StorageProvider, STORAGE_PROVIDER};
        use std::io::{Error, ErrorKind};

        Ok(STORAGE_PROVIDER.load(self.uuid)?)
    }
    pub async fn create_from_original(
        pool: Arc<PgPool>,
        emote_uuid: Uuid,
        content_type: String,
        mut file: File,
    ) -> Result<EmoteImage> {
        let mut file_vec: Vec<u8> = vec![];
        file.read_to_end(&mut file_vec)?;

        // TODO lottie files are not implemented
        let mut inserted_image = sqlx::query_as!(
                EmoteImage,
                "INSERT INTO emote_image (width, height, original, content_type, emote_uuid) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            -1,
            -1,
                true,
                content_type,
                emote_uuid
            )
            .fetch_one(&*pool)
            .await?;

        let proc = ImageProcessor::save(file_vec, inserted_image.uuid, content_type)?;

        // Update the image to say the processing is over
        inserted_image.processing = sqlx::query!(
            "UPDATE emote_image SET processing = ($1), width = ($2), height = ($3) WHERE uuid = ($4) RETURNING processing",
            false,
            proc.image_width as i32,
            proc.image_height as i32,
            inserted_image.uuid
        )
        .fetch_one(&*pool)
        .await?
        .processing;

        for size in [24, 48, 64, 128, 256] {
            Self::resize_image(Arc::clone(&pool), emote_uuid, size, None).await?;
        }

        Ok(inserted_image)
    }

    // If height isn't specified, resize to aspect ratio
    pub async fn resize_image(
        pool: Arc<PgPool>,
        emote_uuid: Uuid,
        width: i32,
        height: Option<i32>, // doesn't work yet
    ) -> Result<bool> {
        actix_web::rt::spawn(async move {
            let orig_emote_image = sqlx::query_as!(
                EmoteImage,
                "SELECT * FROM emote_image WHERE emote_uuid = ($1) AND original = ($2)",
                emote_uuid,
                true
            )
            .fetch_one(&*pool)
            .await?;

            let proc = ImageProcessor::load(orig_emote_image.uuid, orig_emote_image.content_type)?;

            let resized_emote_image = sqlx::query_as!(
                EmoteImage,
                "INSERT INTO emote_image (emote_uuid, width, height, original, content_type, processing) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
                emote_uuid, width, -1, false, proc.image_type_handler.out_extension(), true
            ).fetch_one(&*pool).await?;

            info!("Start resizing image; wait");

            let (new_width, new_height) = proc.resize(
                resized_emote_image.uuid,
                width as u32,
                height.map(|x| x as u32),
                None,
            )?;

            // TODO change the content_type to actually be the extension BASED ON the emote, not the orig image content type
            // any point in setting width?
            let mut transact_fail = false;
            if let Ok(res) = sqlx::query!(
                "UPDATE emote_image SET processing = ($1), width = ($2), height = ($3) WHERE uuid = ($4)",
                false,
                new_width as u32,
                new_height as u32,
                resized_emote_image.uuid,
            )
            .execute(&*pool)
            .await {
                let updated_images = res.rows_affected();
                info!("# of images updated: {}", updated_images);

                if updated_images == 0 {
                    transact_fail = true;
                }
            }
            else {
                transact_fail = true;
            }

            if transact_fail {
                // this is a duplicate, or something went wrong, rollback
                sqlx::query!(
                    "DELETE FROM emote_image WHERE uuid = ($1)",
                    resized_emote_image.uuid
                )
                .execute(&*pool)
                .await?;
                // TODO ADD DELETE FUNCTION TO ImageProcessor
            }

            Ok::<(), async_graphql::Error>(())
        });
        info!("spawned resizing image");

        Ok(true) // guess it always returns true...
    }

    fn emote_path(uuid: Uuid, extension: String) -> Result<PathBuf> {
        let emotes_dir = EMOTES_CONFIG.data_dir.join("emotes");
        if !emotes_dir.exists() {
            std::fs::create_dir_all(&emotes_dir)?
        }

        let abs_emote_path =
            std::fs::canonicalize(emotes_dir)?.join(format!("{}.{}", uuid, extension));
        info!("abs_emote_path is {:?}", abs_emote_path);
        Ok(abs_emote_path)
    }

    // TODO add multiplier
    pub async fn by_emote_and_size(
        pool: Arc<PgPool>,
        emote_uuid: Uuid,
        width: i32,
        height: Option<i32>,
    ) -> Result<Option<EmoteImage>> {
        if let Some(height) = height {
            Ok(sqlx::query_as!(
                EmoteImage,
                "SELECT * FROM emote_image WHERE emote_uuid = ($1) AND width = ($2) AND height = ($3)",
                emote_uuid,
                width,
                height
            )
            .fetch_optional(&*pool)
            .await?)
        } else {
            Ok(sqlx::query_as!(
                EmoteImage,
                "SELECT * FROM emote_image WHERE emote_uuid = ($1) AND width = ($2)",
                emote_uuid,
                width,
            )
            .fetch_optional(&*pool)
            .await?)
        }
    }
}
