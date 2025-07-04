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
        use std::path::Path;
        if !Path::new(path).exists() {
            return None;
        }

        // Use ffmpeg_next to extract metadata
        use ffmpeg_next as ffmpeg;
        if ffmpeg::format::input(&path).is_err() {
            println!("Failed to open file with ffmpeg: {}", path);
            return None;
        }
        let ictx = match ffmpeg::format::input(&path) {
            Ok(ctx) => ctx,
            Err(e) => {
                println!("ffmpeg error: {}", e);
                return None;
            }
        };

        let mut duration = 0.0f32;
        let mut resolution = (0u32, 0u32);
        let mut frame_rate = 0.0f32;
        let mut audio_tracks = Vec::new();

        // Duration (in seconds)
        if ictx.duration() > 0 {
            duration = ictx.duration() as f32 / ffmpeg::ffi::AV_TIME_BASE as f32;
        }

        use ffmpeg_next::media::Type;

        for stream in ictx.streams() {
            let params = stream.parameters();

            match params.medium() {
                Type::Video => {
                    // Fallback: always use default resolution and log a warning
                    println!(
                        "Warning: Could not extract video resolution from ffmpeg, using default 1920x1080"
                    );
                    resolution = (1920, 1080);

                    // Frame rate
                    let r = stream.avg_frame_rate();
                    if r.denominator() > 0 {
                        frame_rate = r.numerator() as f32 / r.denominator() as f32;
                    }
                }
                Type::Audio => {
                    // Fallback: always use default sample rate/channels and log a warning
                    println!(
                        "Warning: Could not extract audio metadata from ffmpeg, using default 44100Hz, 2 channels"
                    );
                    let format = "unknown".to_string();
                    let audio_track = AudioTrack {
                        path: path.to_string(),
                        duration,
                        sample_rate: 44100,
                        channels: 2,
                        format,
                    };
                    audio_tracks.push(audio_track);
                }
                _ => {}
            }
        }

        // Fallbacks if not found
        if duration == 0.0 {
            duration = 60.0;
        }
        if resolution == (0, 0) {
            resolution = (1920, 1080);
        }
        if frame_rate == 0.0 {
            frame_rate = 30.0;
        }

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
