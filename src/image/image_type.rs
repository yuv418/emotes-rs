use crate::image::{resizer_backends::VipsResizerBackend, ResizerBackend};
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum ImageType {
    WEBPAnimated,
    WEBPStill,
    PNG,
    GIF,
    APNG,
    Lottie,
    SVG,
}

pub struct ImageTypeHandler {
    pub image_type: ImageType,
    pub image_resizer: Box<dyn ResizerBackend>,
    pub image_buffer: Arc<Vec<u8>>,
}

impl ImageTypeHandler {
    // this function is not very nice ):
    pub fn from_content_type(content_type: &str, image_buffer: Vec<u8>) -> Result<Option<Self>> {
        let image_buffer = Arc::new(image_buffer);
        let no_frames = match content_type {
            "image/png" | "image/gif" | "image/apng" | "image/webp" => {
                VipsResizerBackend::no_frames(Arc::clone(&image_buffer))
            }
            _ => unimplemented!(),
        }?;

        let mut image_type = match content_type {
            "image/webp" => {
                if no_frames > 1 {
                    ImageType::WEBPStill
                } else {
                    ImageType::WEBPAnimated
                }
            }
            "image/png" => ImageType::PNG,
            "image/gif" => ImageType::GIF,
            "image/apng" => ImageType::APNG,
            "application/json" => ImageType::Lottie,
            "image/svg+xml" => ImageType::SVG,
            _ => {
                return Ok(None);
            }
        };

        let image_resizer: Box<dyn ResizerBackend> = Box::new(match content_type {
            "image/png" | "image/gif" | "image/apng" | "image/webp" => {
                VipsResizerBackend::new(Arc::clone(&image_buffer), image_type)
            }
            _ => unimplemented!(),
        });

        Ok(Some(ImageTypeHandler {
            image_type,
            image_resizer,
            image_buffer,
        }))
    }

    pub fn out_extension(&self) -> String {
        match self.image_type {
            ImageType::PNG => "png",
            ImageType::SVG => "png",
            ImageType::WEBPStill => "png",
            ImageType::WEBPAnimated => "gif",
            ImageType::APNG => "gif",
            ImageType::Lottie => "gif",
            ImageType::GIF => "gif",
        }
        .to_string()
    }
    pub fn in_extension(&self) -> String {
        match self.image_type {
            ImageType::PNG => "png",
            ImageType::SVG => "svg",
            ImageType::WEBPStill => "webp",
            ImageType::WEBPAnimated => "webp",
            ImageType::APNG => "apng",
            ImageType::Lottie => "lottie", // even though they're json, this makes more sense
            ImageType::GIF => "gif",
        }
        .to_string()
    }
}
