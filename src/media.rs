#[derive(Debug, Clone)]
pub enum Media {
    Video(String),
    Audio(String),
    Effect(String),
    Image(String),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct AudioTrack {
    pub path: String,
    pub duration: f32,
    pub sample_rate: u32,
    pub channels: u32,
    pub format: String,
}

#[derive(Clone, Debug)]
pub struct VideoFile {
    pub path: String,
    pub duration: f32,
    pub resolution: (u32, u32),
    pub frame_rate: f32,
    pub audio_tracks: Vec<AudioTrack>,
}

impl VideoFile {
    pub fn from_path(path: &str) -> Option<Self> {
        use gstreamer as gst;
        use gstreamer::prelude::*;
        use gstreamer_video as gst_video;
        use std::path::Path;
        use std::sync::{Arc, Mutex};

        if !Path::new(path).exists() {
            return None;
        }

        // Try to use GStreamer to extract video metadata
        if let Some(video_file) = Self::extract_metadata_with_gstreamer(path) {
            return Some(video_file);
        }

        // Fallback: create video file with default metadata
        println!(
            "Warning: Could not extract metadata, using defaults for {}",
            path
        );
        Some(VideoFile {
            path: path.to_string(),
            duration: 60.0,           // Default duration (increased from 10s)
            resolution: (1920, 1080), // Default resolution
            frame_rate: 30.0,         // Default frame rate
            audio_tracks: Vec::new(),
        })
    }

    fn extract_metadata_with_gstreamer(path: &str) -> Option<VideoFile> {
        use gstreamer as gst;
        use gstreamer::prelude::*;
        use gstreamer_video as gst_video;
        use std::sync::{Arc, Mutex};

        println!("Extracting metadata from: {}", path);

        let pipeline = gst::Pipeline::new();
        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", path)
            .build()
            .ok()?;
        let decodebin = gst::ElementFactory::make("decodebin").build().ok()?;

        pipeline.add_many([&filesrc, &decodebin]).ok()?;
        gst::Element::link_many([&filesrc, &decodebin]).ok()?;

        let video_info_arc = Arc::new(Mutex::new(None::<gst_video::VideoInfo>));
        let video_info_clone = video_info_arc.clone();
        let audio_tracks_arc = Arc::new(Mutex::new(Vec::<AudioTrack>::new()));
        let audio_tracks_clone = audio_tracks_arc.clone();

        let path_owned = path.to_string();
        decodebin.connect_pad_added(move |_decodebin, pad| {
            if let Some(caps) = pad.current_caps() {
                let structure = caps.structure(0).unwrap();
                let name = structure.name();

                if name.as_str().starts_with("video/") {
                    if let Ok(info) = gst_video::VideoInfo::from_caps(&caps) {
                        let width = info.width();
                        let height = info.height();
                        if let Ok(mut guard) = video_info_clone.lock() {
                            *guard = Some(info);
                        }
                        println!("Found video stream: {}x{}", width, height);
                    }
                } else if name.as_str().starts_with("audio/") {
                    // Extract audio information
                    let sample_rate = structure.get::<i32>("rate").unwrap_or(44100);
                    let channels = structure.get::<i32>("channels").unwrap_or(2);
                    let format = structure.name().as_str().to_string();

                    let audio_track = AudioTrack {
                        path: path_owned.clone(),
                        duration: 0.0, // Will be set later
                        sample_rate: sample_rate as u32,
                        channels: channels as u32,
                        format,
                    };

                    if let Ok(mut guard) = audio_tracks_clone.lock() {
                        guard.push(audio_track);
                    }
                    println!(
                        "Found audio stream: {}Hz, {} channels",
                        sample_rate, channels
                    );
                }
            }
        });

        // Set pipeline to paused to get metadata
        if pipeline.set_state(gst::State::Paused).is_err() {
            println!("Failed to set pipeline to paused");
            return None;
        }

        // Wait for async done or error
        let bus = pipeline.bus().unwrap();
        if let Some(msg) = bus.timed_pop_filtered(
            gst::ClockTime::from_seconds(5),
            &[gst::MessageType::AsyncDone, gst::MessageType::Error],
        ) {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    println!("Pipeline error: {}", err.error());
                    let _ = pipeline.set_state(gst::State::Null);
                    return None;
                }
                _ => {}
            }
        }

        let mut duration = 60.0; // Default 60 seconds instead of 10
        let mut resolution = (1920, 1080);
        let mut frame_rate = 30.0;
        let mut audio_tracks = Vec::new();

        // Query duration
        if let Some(dur) = pipeline.query_duration::<gst::ClockTime>() {
            let seconds = dur.seconds() as f32;
            if seconds > 0.1 {
                // Ensure we got a valid duration
                duration = seconds;
                println!("Video duration: {}s", duration);
            } else {
                println!(
                    "Warning: Invalid duration detected ({}s), using default",
                    seconds
                );
            }
        } else {
            println!("Warning: Could not query duration, using default");
        }

        // Get video info
        if let Ok(guard) = video_info_arc.lock() {
            if let Some(info) = guard.as_ref() {
                resolution = (info.width(), info.height());
                let fps = info.fps();
                if fps.denom() > 0 {
                    frame_rate = fps.numer() as f32 / fps.denom() as f32;
                }
                println!(
                    "Video resolution: {}x{}, fps: {}",
                    resolution.0, resolution.1, frame_rate
                );
            }
        }

        // Get audio tracks
        if let Ok(mut guard) = audio_tracks_arc.lock() {
            for track in guard.iter_mut() {
                track.duration = duration;
            }
            audio_tracks = guard.clone();
        }

        let _ = pipeline.set_state(gst::State::Null);

        Some(VideoFile {
            path: path.to_string(),
            duration,
            resolution,
            frame_rate,
            audio_tracks,
        })
    }
}

#[derive(Default)]
pub struct MediaLibrary {
    pub videos: Vec<VideoFile>,
    pub audios: Vec<AudioFile>,
}

pub struct AudioFile {
    pub path: String,
    pub duration: f32,
}
