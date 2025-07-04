use crate::frame::Frame;

/// Stub for a future ffmpeg_next-based video decoder.
/// All GStreamer code has been removed.
pub struct VideoDecoder {}

impl VideoDecoder {
    pub fn new(_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement using ffmpeg_next
        Err("VideoDecoder is not yet implemented. All GStreamer code has been removed.".into())
    }
}
