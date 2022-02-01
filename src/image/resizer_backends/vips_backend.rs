use crate::image::{ImageType, ResizerBackend};
use anyhow::Result;
use libvips::{ops, VipsImage};
use log::info;
use std::sync::Arc;

pub struct VipsResizerBackend {
    in_buffer: Arc<Vec<u8>>,
    in_type: ImageType,
}

impl VipsResizerBackend {
    fn vips_image(&self) -> Result<VipsImage> {
        Ok(match self.in_type {
            ImageType::WEBPAnimated | ImageType::WEBPStill => ops::webpload_buffer(&self.in_buffer),
            ImageType::GIF => VipsImage::new_from_buffer(&self.in_buffer, "[n=-1]"),
            ImageType::JPEG | ImageType::PNG => VipsImage::new_from_buffer(&self.in_buffer, ""),
            ImageType::SVG => ops::svgload_buffer(&self.in_buffer),
            _ => unimplemented!(), // libvips doesn't support apngs, nor will lottie happen here
        }?)
    }
}

impl ResizerBackend for VipsResizerBackend {
    fn new(in_buffer: Arc<Vec<u8>>, in_type: ImageType) -> Self {
        Self { in_buffer, in_type }
    }
    fn resize(
        &self,
        out_width: u32,
        out_height: Option<u32>,
        out_multiplier: Option<u32>,
    ) -> Result<(u32, u32, Vec<u8>)> {
        let vips_image = self.vips_image()?;
        // TODO use the height
        if let Some(_) = out_height {
            unimplemented!()
        }
        let resized_vips_image = ops::thumbnail_image(&vips_image, out_width as i32)?;

        Ok((
            resized_vips_image.get_width() as u32,
            resized_vips_image.get_page_height() as u32,
            match self.in_type {
                ImageType::WEBPAnimated | ImageType::GIF => {
                    ops::gifsave_buffer(&resized_vips_image)?
                }
                ImageType::PNG | ImageType::JPEG | ImageType::SVG => {
                    ops::pngsave_buffer(&resized_vips_image)?
                }
                _ => unimplemented!(),
            },
        ))
    }

    fn dimensions(&self) -> Result<(u32, u32)> {
        let vips_image = self.vips_image()?;
        info!("vips image's height is {}", vips_image.get_page_height());
        Ok((
            vips_image.get_width() as u32,
            vips_image.get_page_height() as u32,
        ))
    }

    fn no_frames(in_buffer: Arc<Vec<u8>>) -> Result<u32> {
        // the more I think about this, the more this is a hack
        let frame_counter = VipsImage::new_from_buffer(&in_buffer, "[n=-1]"); // load all frames

        if let Err(_) = frame_counter {
            return Ok(1);
        }
        Ok(frame_counter.unwrap().get_n_pages() as u32)
    }
}
