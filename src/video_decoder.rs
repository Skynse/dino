use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use std::sync::{Arc, Mutex};

use crate::frame::Frame;

pub struct VideoDecoder {
    pipeline: gst::Pipeline,
    appsink: gst_app::AppSink,
    width: u32,
    height: u32,
    current_frame: Arc<Mutex<Option<Frame>>>,
    duration: Option<gst::ClockTime>,
}

impl VideoDecoder {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        println!("Creating VideoDecoder for: {}", path);

        // Initialize GStreamer if it hasn't been already
        gst::init()?;

        // Verify file exists before creating pipeline
        let path_obj = std::path::Path::new(path);
        if !path_obj.exists() {
            println!("Video file does not exist: {}", path);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Video file not found: {}", path),
            )));
        }

        // Additional safety check for file access
        match std::fs::File::open(path) {
            Ok(_) => println!("File is accessible: {}", path),
            Err(e) => {
                println!("Cannot open video file: {} - Error: {}", path, e);
                return Err(Box::new(e));
            }
        }

        // Check if file size is valid
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() == 0 {
                println!("Video file is empty: {}", path);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Empty video file",
                )));
            }
        }

        // Create a simple pipeline with filesrc -> decodebin -> videoconvert -> videoscale -> appsink
        let pipeline_str = format!(
            "filesrc location=\"{}\" ! decodebin name=decode ! queue max-size-buffers=30 ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=640,height=480 ! appsink name=sink sync=false max-buffers=1 drop=true",
            path.replace("\\", "/")
        );

        let pipeline = gst::parse::launch(&pipeline_str)?;
        let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();

        // Get the appsink element from the pipeline
        let appsink = pipeline
            .by_name("sink")
            .expect("Couldn't get appsink element")
            .downcast::<gst_app::AppSink>()
            .expect("Element is not an AppSink");

        let current_frame = Arc::new(Mutex::new(None));
        let frame_clone = current_frame.clone();

        // Set up sample callback
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let caps = sample.caps().ok_or(gst::FlowError::Error)?;

                    let info =
                        gst_video::VideoInfo::from_caps(caps).map_err(|_| gst::FlowError::Error)?;

                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let data = map.as_slice();

                    println!(
                        "Got frame: {}x{}, {} bytes",
                        info.width(),
                        info.height(),
                        data.len()
                    );

                    // Verify we have valid data in the expected format
                    let expected_size = (info.width() * info.height() * 4) as usize;
                    if data.len() != expected_size {
                        println!(
                            "Invalid frame data size: got {} bytes, expected {}",
                            data.len(),
                            expected_size
                        );
                        return Ok(gst::FlowSuccess::Ok);
                    }

                    let frame = Frame {
                        width: info.width(),
                        height: info.height(),
                        pixels: data.to_vec(),
                        timestamp: 0.0,
                    };

                    if let Ok(mut current) = frame_clone.lock() {
                        *current = Some(frame);
                    }

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        // Set pipeline to paused to preroll
        pipeline.set_state(gst::State::Paused)?;

        // Wait for preroll with a reasonable timeout
        let bus = pipeline.bus().unwrap();
        for _ in 0..50 {
            // Try for 5 seconds (50 * 100ms)
            if let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(100)) {
                match msg.view() {
                    gst::MessageView::AsyncDone(_) => {
                        println!("Pipeline prerolled successfully");
                        break;
                    }
                    gst::MessageView::Error(err) => {
                        println!("Error during preroll: {}", err.error());
                        pipeline.set_state(gst::State::Null)?;
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("GStreamer error: {}", err.error()),
                        )));
                    }
                    gst::MessageView::Eos(_) => {
                        println!("End of stream during preroll");
                        break;
                    }
                    _ => continue,
                }
            }
        }

        // Query duration
        let duration = pipeline.query_duration::<gst::ClockTime>();
        if let Some(dur) = duration {
            println!("Video duration: {}s", dur.seconds());
        } else {
            println!("Couldn't determine video duration");
        }

        Ok(VideoDecoder {
            pipeline,
            appsink,
            width: 640,
            height: 480,
            current_frame,
            duration,
        })
    }

    pub fn get_frame_at_time(&mut self, time: f32) -> Option<Frame> {
        // Validate input
        if time < 0.0 {
            println!("Invalid time request: {}", time);
            return None;
        }

        println!("Seeking to time: {}s", time);

        // Check if time is within video duration
        if let Some(duration) = self.duration {
            if time > duration.seconds() as f32 {
                println!(
                    "Requested time {}s exceeds video duration {}s",
                    time,
                    duration.seconds()
                );
                return None;
            }
        }

        // Clear any previous frame
        if let Ok(mut frame_guard) = self.current_frame.lock() {
            *frame_guard = None;
        }

        // Flush pipeline and pause
        let _ = self.pipeline.set_state(gst::State::Paused);
        std::thread::sleep(std::time::Duration::from_millis(30));

        // Create accurate seek event
        let seek_time = gst::ClockTime::from_nseconds((time * 1_000_000_000.0) as u64);
        let seek_event = gst::event::Seek::new(
            1.0,
            gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
            gst::SeekType::Set,
            seek_time,
            gst::SeekType::None,
            gst::ClockTime::NONE,
        );

        // Send seek event with retry logic
        let mut seek_success = false;
        for i in 0..3 {
            // Try up to 3 times
            if self.pipeline.send_event(seek_event.clone()) {
                seek_success = true;
                break;
            }
            println!("Seek attempt {} failed, retrying...", i + 1);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        if !seek_success {
            println!("Failed to send seek event after multiple attempts");
            return None;
        }

        // Set to playing to process the seek
        if self.pipeline.set_state(gst::State::Playing).is_err() {
            println!("Failed to set pipeline to playing");
            return None;
        }

        // Give the pipeline time to process the seek
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Wait for the frame to be processed with longer timeout
        let bus = self.pipeline.bus().unwrap();
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(1000); // 1 second timeout

        while start_time.elapsed() < timeout {
            // Check for messages
            while let Some(msg) = bus.pop() {
                match msg.view() {
                    gst::MessageView::Eos(_) => {
                        println!("End of stream reached");
                        break;
                    }
                    gst::MessageView::Error(err) => {
                        println!("Pipeline error: {}", err.error());
                        let _ = self.pipeline.set_state(gst::State::Paused);
                        return None;
                    }
                    gst::MessageView::StateChanged(state_changed) => {
                        if state_changed
                            .src()
                            .map(|s| s == self.pipeline.upcast_ref::<gst::Object>())
                            == Some(true)
                        {
                            let (old, new) = (state_changed.old(), state_changed.current());
                            println!("Pipeline state changed from {:?} to {:?}", old, new);
                        }
                    }
                    _ => {}
                }
            }

            // Check if we have a frame
            if let Ok(mut frame_guard) = self.current_frame.try_lock() {
                if let Some(mut frame) = frame_guard.take() {
                    frame.timestamp = time;
                    println!("Successfully extracted frame at {}s", time);

                    // Pause pipeline after getting frame
                    let _ = self.pipeline.set_state(gst::State::Paused);
                    return Some(frame);
                }
            }

            // Shorter sleep to not block the main thread too much
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        println!("Timeout waiting for frame at {}s", time);

        // Try to recover by resetting pipeline state
        let _ = self.pipeline.set_state(gst::State::Null);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = self.pipeline.set_state(gst::State::Ready);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = self.pipeline.set_state(gst::State::Paused);

        // Return None to indicate failure
        None
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        println!("Cleaning up VideoDecoder resources");

        // Send EOS event to flush any pending data
        let _ = self.pipeline.send_event(gst::event::Eos::new());

        // Wait a moment for the EOS to propagate
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Set to null state to free resources
        let _ = self.pipeline.set_state(gst::State::Null);

        println!("VideoDecoder resources cleaned up");
    }
}
