use anyhow::Result;

use crate::image::ImageType;
use std::sync::Arc;

pub trait ResizerBackend {
    fn new(in_buffer: Arc<Vec<u8>>, in_type: ImageType) -> Self
    where
        Self: Sized;

    // width, height, data array
    fn resize(
        &self,
        out_width: u32,
        out_height: Option<u32>,
        out_multiplier: Option<u32>,
    ) -> Result<(u32, u32, Vec<u8>)>;

    fn dimensions(&self) -> Result<(u32, u32)>;
    fn no_frames(in_buffer: Arc<Vec<u8>>) -> Result<u32>
    where
        Self: Sized; // separate since circular dependency on no_frames to create new ResizerBackend otherwise
}

mod vips_backend;

pub use vips_backend::VipsResizerBackend;
