use crate::image::{ImageType, ResizerBackend};
use anyhow::Result;
use std::sync::Arc;

pub struct VipsResizerBackend {
    in_buffer: Arc<Vec<u8>>,
    in_type: ImageType,
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
        unimplemented!()
    }

    fn dimensions(&self) -> Result<(u32, u32)> {
        unimplemented!()
    }

    fn no_frames(in_buffer: Arc<Vec<u8>>) -> Result<u32> {
        unimplemented!()
    }
}
