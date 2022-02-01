use anyhow::Result;
use uuid::Uuid;

use crate::{
    config::EMOTES_CONFIG,
    image::{ImageType, ImageTypeHandler, ResizerBackend},
    storage::{LocalStorageProvider, StorageProvider, STORAGE_PROVIDER},
};

// metadata about source image
pub struct ImageProcessor {
    pub image_width: u32,
    pub image_height: u32,
    pub image_type_handler: ImageTypeHandler,
    pub image_uuid: Uuid,
}

impl ImageProcessor {
    pub fn save(
        image_buffer: Vec<u8>,
        image_uuid: Uuid,
        image_content_type: String,
    ) -> Result<Self> {
        // TODO don't unwrap
        let image_type_handler =
            ImageTypeHandler::from_content_type(&image_content_type, image_buffer)?.unwrap(); // temporary hack for getting resizer
        let (image_width, image_height) = image_type_handler.image_resizer.dimensions()?;

        // in_extension for "input" extension since this function is for a "source" or "original" file
        STORAGE_PROVIDER.save(image_uuid, &image_type_handler.image_buffer)?;

        Ok(Self {
            image_width,
            image_height,
            image_type_handler,
            image_uuid,
        })
    }

    pub fn load(image_uuid: Uuid, image_content_type: String) -> Result<Self> {
        // in_extension for "input" extension since this function is for a "source" or "original" file

        let image_buffer = STORAGE_PROVIDER.load(image_uuid)?;

        let image_type_handler =
            ImageTypeHandler::from_content_type(&image_content_type, image_buffer)?.unwrap(); // temporary hack for getting resizer
        let (image_width, image_height) = image_type_handler.image_resizer.dimensions()?;

        Ok(Self {
            image_width,
            image_height,
            image_type_handler,
            image_uuid,
        })
    }

    // width, height
    pub fn resize(
        &self,
        out_uuid: Uuid,
        out_width: u32,
        out_height: Option<u32>,
        out_multiplier: Option<u32>,
    ) -> Result<(u32, u32)> {
        let (proc_out_width, proc_out_height, proc_out_image_bytes) = self
            .image_type_handler
            .image_resizer
            .resize(out_width, out_height, out_multiplier)?;

        STORAGE_PROVIDER.save(out_uuid, &proc_out_image_bytes)?;

        Ok((proc_out_width, proc_out_height))
    }
}
