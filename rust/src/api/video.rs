use ffmpeg_next as ffmpeg;
use std::fs;
use std::path::Path;

#[flutter_rust_bridge::frb(sync)]
pub fn extract_frames_to_disk(
    video_path: String,
    output_dir: String,
    every_nth: usize,
    max_frames: usize,
) -> Result<Vec<String>, String> {
    ffmpeg::init().map_err(|e| format!("FFmpeg init error: {:?}", e))?;

    let mut ictx =
        ffmpeg::format::input(&video_path).map_err(|e| format!("Failed to open video: {:?}", e))?;

    let input = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;
    let video_stream_index = input.index();

    let mut decoder = input
        .codec()
        .decoder()
        .video()
        .map_err(|e| format!("Failed to get video decoder: {:?}", e))?;

    let mut frame_index = 0;
    let mut saved = 0;
    let mut saved_paths = Vec::new();

    fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create output dir: {:?}", e))?;

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder
                .send_packet(&packet)
                .map_err(|e| format!("Send packet error: {:?}", e))?;
            let mut frame = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                if frame_index % every_nth == 0 && saved < max_frames {
                    let file_name = format!("{}/frame_{:05}.png", output_dir, frame_index);
                    save_frame_as_png(&frame, &file_name)
                        .map_err(|e| format!("Failed to save frame: {:?}", e))?;
                    saved_paths.push(file_name);
                    saved += 1;
                }
                frame_index += 1;
                if saved >= max_frames {
                    break;
                }
            }
        }
        if saved >= max_frames {
            break;
        }
    }
    Ok(saved_paths)
}

// Helper function to save a frame as PNG
fn save_frame_as_png(frame: &ffmpeg::util::frame::video::Video, path: &str) -> Result<(), String> {
    use image::{ImageBuffer, Rgba};
    let width = frame.width();
    let height = frame.height();
    let data = frame.data(0);

    // You may need to convert pixel format here for real-world videos!
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data.to_vec())
        .ok_or("Failed to create image buffer")?;
    buffer
        .save(path)
        .map_err(|e| format!("Failed to save PNG: {:?}", e))
}
