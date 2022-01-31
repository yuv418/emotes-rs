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

use crate::config::EMOTES_CONFIG;

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
    pub fn get_file(&self) -> anyhow::Result<File> {
        use std::io::{Error, ErrorKind};
        let emote_path = Self::emote_path(
            self.uuid,
            Self::content_type_to_extension(&self.content_type, false).map_err(|x| {
                Error::new(
                    ErrorKind::Other,
                    "Failed to convert emote image's content type to its extension for reading",
                )
            })?,
        )
        .map_err(|x| Error::new(ErrorKind::Other, "Failed to get path for emote"))?;

        Ok(File::open(&emote_path)?)
    }

    fn vips_get_width_height(buf: &[u8]) -> Result<(i32, i32)> {
        let image = VipsImage::new_from_buffer(&buf, "")?; // the options are for transparent GIFs
        Ok((image.get_width(), image.get_height()))
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
        let extension = Self::content_type_to_extension(&content_type, true)?;
        info!("choose ext {}", extension);

        let (width, height) = Self::vips_get_width_height(&file_vec)?;
        let mut inserted_image = sqlx::query_as!(
                EmoteImage,
                "INSERT INTO emote_image (width, height, original, content_type, emote_uuid) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            width,
            height,
                true,
                content_type,
                emote_uuid
            )
            .fetch_one(&*pool)
            .await?;

        Self::save_vips_image(&file_vec, inserted_image.uuid, extension)?;

        // Update the image to say the processing is over
        inserted_image.processing = sqlx::query!(
            "UPDATE emote_image SET processing = ($1) WHERE uuid = ($2) RETURNING processing",
            false,
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

            // get file
            let path_for_image = Self::emote_path(
                orig_emote_image.uuid,
                Self::content_type_to_extension(&orig_emote_image.content_type, true)
                    .expect("failed to get emote path"),
            )?;

            let ext =
                Self::content_type_to_extension(&orig_emote_image.content_type, false).unwrap();
            info!("output ext is {}", ext);
            let ext_content_type = match ext.as_str() {
                "gif" => "image/gif",
                "png" => "image/png",
                _ => return Err("webp upload is unsupported at the moment".into()), // TODO use enums so this isn't a thing
            };
            let resized_emote_image = sqlx::query_as!(
                EmoteImage,
                "INSERT INTO emote_image (emote_uuid, width, height, original, content_type, processing) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
                emote_uuid, width, -1, false, ext_content_type, true
            ).fetch_one(&*pool).await?;

            info!("Start resizing image; wait");

            let (out_path, new_width, new_height) = if let Some(height) = height {
                unimplemented!("height rescale not implemented")
            } else {
                Self::vips_image_resize(
                    path_for_image,
                    resized_emote_image.uuid,
                    ext,
                    width,
                    height,
                )
            };

            // TODO change the content_type to actually be the extension BASED ON the emote, not the orig image content type
            // any point in setting width?
            let mut transact_fail = false;
            if let Ok(res) = sqlx::query!(
                "UPDATE emote_image SET processing = ($1), width = ($2), height = ($3) WHERE uuid = ($4)",
                false,
                new_width,
                new_height,
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

                // Delete the failed emote file
                std::fs::remove_file(out_path)?;
            }

            Ok::<(), async_graphql::Error>(())
        });
        info!("spawned resizing image");

        Ok(true) // guess it always returns true...
    }

    fn vips_image_resize(
        orig_path: PathBuf,
        resized_uuid: Uuid,
        ext: String,
        width: i32,
        height: Option<i32>,
    ) -> (PathBuf, i32, i32) {
        // looping is set by default
        let vips_opts = if ext == "gif" { "[n=-1]" } else { "" };
        let orig_vips_image =
            VipsImage::new_from_file(&(orig_path.to_str().unwrap().to_owned() + vips_opts))
                .unwrap();
        let resized_vips_image =
            ops::thumbnail_image(&orig_vips_image, width).expect("failed to resize image!");

        let width = resized_vips_image.get_width();
        let height = resized_vips_image.get_page_height(); // gifs have a large regular height, we want to use the page (frame?) height

        let path =
            Self::emote_path(resized_uuid, ext).expect("failed to generate path for resized image");

        // gif looping will only work with a forked libvips, as libvips lacks bindings to `vips_image_set`.

        resized_vips_image
            .image_write_to_file(path.to_str().unwrap())
            .expect("failed to write resized image");

        (path, width, height)
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

    fn save_vips_image(vips_image_bytes: &[u8], uuid: Uuid, extension: String) -> Result<()> {
        // there are bigger issues if that unwrap fails
        use std::fs;

        let path = Self::emote_path(uuid, extension)?;
        info!("WRITING IMAGE TO PATH {:?}", path);
        fs::write(path, vips_image_bytes)?;
        Ok(())
    }

    fn content_type_to_extension(content_type: &str, input: bool) -> Result<String> {
        let accepted_content_types = {
            let mut i = HashMap::new();
            i.insert("image/png", ("png", "png"));
            i.insert("image/apng", ("apng", "gif"));
            i.insert("image/gif", ("gif", "gif"));
            i.insert("image/jpeg", ("jpeg", "png"));
            i.insert("image/svg+xml", ("svg", "png"));
            i.insert("image/webp", ("webp", "webp")); // this one really depends whether it's animated or not, return webp for handling of that
            i.insert("application/json", ("lottie", "gif"));
            i
        };

        if accepted_content_types.contains_key(content_type) {
            let ct_val = accepted_content_types.get(&content_type).unwrap();
            let ct_out = if input { ct_val.0 } else { ct_val.1 };

            Ok(ct_out.to_string())
        } else {
            Err("invalid content type for emote".into())
        }
    }
}
