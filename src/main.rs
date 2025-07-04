mod frame;
mod frame_cache;
mod media;
mod playback;

mod settings;
mod timeline;
mod video_decoder;

use eframe::egui::{self, Id};
use std::path::Path;

use frame::Frame;
use frame_cache::FrameCache;
use media::{Media, MediaLibrary, VideoFile};
use playback::PlaybackState;

use settings::EditorSettings;
use timeline::{DraggedMedia, TimeLine};

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Video Editor",
        native_options,
        Box::new(|cc| Ok(Box::new(VideoEditorApp::new(cc)))),
    );
}

fn create_test_frame(width: u32, height: u32, timestamp: f32) -> Frame {
    assert!(width > 0 && height > 0, "Frame dimensions must be positive");
    let pixel_count = (width * height) as usize;
    let mut pixels = vec![0; pixel_count * 4];

    // Create a gradient based on timestamp
    for i in 0..pixel_count {
        let x = i % width as usize;
        let y = i / width as usize;
        let r = ((x as f32 / width as f32) * 255.0 * (timestamp * 0.1).sin().abs()) as u8;
        let g = ((y as f32 / height as f32) * 255.0 * (timestamp * 0.2).cos().abs()) as u8;
        let b = ((timestamp * 50.0).sin().abs() * 255.0) as u8;
        pixels[i * 4] = r;
        pixels[i * 4 + 1] = g;
        pixels[i * 4 + 2] = b;
        pixels[i * 4 + 3] = 255; // alpha
    }

    Frame {
        width,
        height,
        timestamp,
        pixels,
    }
}

fn create_test_clip() -> timeline::Clip {
    timeline::Clip {
        start_time: 0.0,
        duration: 10.0,
        media: Media::Video("test_video".to_string()),
        is_being_dragged: false,
        drag_offset: 0.0,
    }
}

struct VideoEditorApp {
    timeline: TimeLine,
    playback: PlaybackState,
    media_library: MediaLibrary,
    settings: EditorSettings,
    preview_frame: Option<Frame>,
    frame_cache: FrameCache,
}

impl VideoEditorApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            timeline: TimeLine::default(),
            playback: PlaybackState::default(),
            media_library: MediaLibrary::default(),
            settings: EditorSettings::default(),
            preview_frame: None,
            frame_cache: FrameCache::new(),
        }
    }

    fn update_preview_frame(&mut self) {
        // Find the active clip at current timeline position
        self.preview_frame = None;

        // Create a temporary vector to avoid borrowing issues
        let mut active_clip: Option<(String, f32)> = None;

        for track in &self.timeline.tracks {
            for clip in &track.clips {
                let clip_start = clip.start_time as f32;
                let clip_end = (clip.start_time + clip.duration) as f32;

                if self.timeline.current_time >= clip_start && self.timeline.current_time < clip_end
                {
                    // Calculate the relative time within the clip
                    let relative_time = self.timeline.current_time - clip_start;

                    match &clip.media {
                        Media::Video(video_path) => {
                            // Ensure relative_time doesn't exceed the actual video duration
                            let clamped_time = relative_time.min(clip.duration as f32 - 0.1);
                            active_clip = Some((video_path.clone(), clamped_time));
                            break;
                        }
                        _ => {} // Handle other media types later
                    }
                }
            }
            if active_clip.is_some() {
                break;
            }
        }

        if let Some((video_path, relative_time)) = active_clip {
            // Always get a frame - either cached or fallback
            let frame = self
                .decode_frame_at_time(&video_path, relative_time)
                .unwrap_or_else(|| self.create_video_frame(&video_path, relative_time, 640, 480));
            self.preview_frame = Some(frame);
        }
    }

    fn decode_frame_at_time(&mut self, video_path: &str, time: f32) -> Option<Frame> {
        // Only fallback frame logic remains
        if !std::path::Path::new(video_path).exists() {
            println!("WARNING: File does not exist: {}", video_path);
            return Some(self.create_video_frame(video_path, time, 640, 480));
        }
        Some(self.create_video_frame(video_path, time, 640, 480))
    }

    fn create_video_frame(&self, video_path: &str, time: f32, width: u32, height: u32) -> Frame {
        println!(
            "Creating fallback frame for {} at time {} with dimensions {}x{}",
            video_path, time, width, height
        );
        // Create a professional-looking placeholder frame
        let pixel_count = (width * height) as usize;
        let mut pixels = vec![0; pixel_count * 4];

        println!("Allocated {} pixels in buffer", pixel_count);

        // Get the filename for display (unused currently but kept for future labeling)
        let _filename = Path::new(video_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        // Create a dark gray background
        let bg_color = 40u8; // Dark gray background

        for i in 0..pixel_count {
            pixels[i * 4] = bg_color; // R
            pixels[i * 4 + 1] = bg_color; // G
            pixels[i * 4 + 2] = bg_color; // B
            pixels[i * 4 + 3] = 255; // Alpha
        }

        // Add a simple progress bar at the bottom
        let bar_height = height as usize / 20;
        let bar_y_start = height as usize - bar_height * 2;
        let progress = (time % 5.0) / 5.0; // Cycles every 5 seconds
        let bar_width = (width as f32 * progress) as usize;

        for y in bar_y_start..bar_y_start + bar_height {
            for x in 0..bar_width {
                let idx = (y * width as usize + x) * 4;
                pixels[idx] = 100; // R
                pixels[idx + 1] = 180; // G
                pixels[idx + 2] = 240; // B
                pixels[idx + 3] = 255; // Alpha
            }
        }

        Frame {
            width,
            height,
            timestamp: time,
            pixels,
        }
    }

    fn add_test_clip_to_timeline(&mut self) {
        let test_clip = create_test_clip();

        // Ensure we have at least one track
        if self.timeline.tracks.is_empty() {
            self.timeline.tracks.push(timeline::Track::default());
        }

        self.timeline.tracks[0].clips.push(test_clip);
    }

    fn add_media_to_timeline(
        &mut self,
        video_file: &VideoFile,
        track_index: usize,
        start_time: f64,
    ) {
        let video_clip = timeline::Clip {
            start_time,
            duration: video_file.duration as f64,
            media: Media::Video(video_file.path.clone()),
            is_being_dragged: false,
            drag_offset: 0.0,
        };

        // Ensure we have enough tracks for video
        while self.timeline.tracks.len() <= track_index {
            self.timeline.tracks.push(timeline::Track::default());
        }

        self.timeline.tracks[track_index].clips.push(video_clip);

        // For the simplified frame cache, we don't do background preloading
        // Frames will be loaded on-demand when they're needed

        // Auto-import audio tracks if they exist
        for (audio_index, audio_track) in video_file.audio_tracks.iter().enumerate() {
            let audio_track_index = track_index + 1 + audio_index;

            // Ensure we have enough audio tracks
            while self.timeline.tracks.len() <= audio_track_index {
                self.timeline.tracks.push(timeline::Track {
                    kind: timeline::TrackKind::Audio,
                    clips: Vec::new(),
                });
            }

            let audio_clip = timeline::Clip {
                start_time,
                duration: audio_track.duration as f64,
                media: Media::Audio(audio_track.path.clone()),
                is_being_dragged: false,
                drag_offset: 0.0,
            };

            self.timeline.tracks[audio_track_index]
                .clips
                .push(audio_clip);
        }
    }

    fn get_effective_video_path(&self, original_path: &str) -> String {
        original_path.to_string()
    }

    fn render_preview_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Preview");
            ui.separator();

            if let Some(frame) = &self.preview_frame {
                let texture_id = ctx.load_texture(
                    "preview_texture",
                    egui::ColorImage::from_rgba_unmultiplied(
                        [frame.width as usize, frame.height as usize],
                        &frame.pixels,
                    ),
                    egui::TextureOptions::LINEAR,
                );

                ui.add_space(10.0);
                let available_size = ui.available_size();
                let image_aspect = frame.width as f32 / frame.height as f32;
                let panel_aspect = available_size.x / available_size.y;

                let (display_width, display_height) = if image_aspect > panel_aspect {
                    let width = available_size.x - 20.0;
                    (width, width / image_aspect)
                } else {
                    let height = available_size.y - 60.0;
                    (height * image_aspect, height)
                };

                ui.add(
                    egui::Image::from_texture(&texture_id)
                        .fit_to_exact_size(egui::vec2(display_width, display_height)),
                );

                ui.add_space(10.0);
                ui.separator();

                // Show detailed video information
                ui.horizontal(|ui| {
                    ui.label(format!("Time: {:.2}s", self.timeline.current_time));
                    ui.separator();
                    ui.label(format!("Resolution: {}x{}", frame.width, frame.height));
                    ui.separator();
                    ui.label(format!("Frame: {:.1}fps", frame.timestamp));
                });

                // Show active clip information
                if let Some(active_clip) = self.get_active_clip_info() {
                    ui.horizontal(|ui| {
                        ui.label("📹");
                        ui.label(format!("Clip: {}", active_clip.0));
                        ui.separator();
                        ui.label(format!("Duration: {:.1}s", active_clip.1));
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No frame to preview");
                });
            }
        });
    }

    fn get_active_clip_info(&self) -> Option<(String, f64)> {
        for track in &self.timeline.tracks {
            for clip in &track.clips {
                let clip_start = clip.start_time as f32;
                let clip_end = (clip.start_time + clip.duration) as f32;

                if self.timeline.current_time >= clip_start && self.timeline.current_time < clip_end
                {
                    if let Media::Video(video_path) = &clip.media {
                        let filename = std::path::Path::new(video_path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        return Some((filename, clip.duration));
                    }
                }
            }
        }
        None
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import Media").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Video files", &["mp4", "avi", "mov", "mkv"])
                            .pick_file()
                        {
                            let path_str = path.to_string_lossy().to_string();
                            if let Some(video_file) = VideoFile::from_path(&path_str) {
                                self.media_library.videos.push(video_file.clone());
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Settings").clicked() {
                        // Open settings dialog would go here
                        ui.close_menu();
                    }
                    if ui.button("Clear Timeline").clicked() {
                        self.timeline.tracks.clear();
                        self.timeline.selected_clip = None;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Export").clicked() {
                        // Handle export
                        ui.close_menu();
                    }
                    if ui.button("Add Test Clip").clicked() {
                        self.add_test_clip_to_timeline();
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn render_side_panels(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("media_library").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Media Library");

                // Add import button to the header
                if ui.button("➕ Import").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Video files", &["mp4", "avi", "mov", "mkv"])
                        .pick_file()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        if let Some(video_file) = VideoFile::from_path(&path_str) {
                            self.media_library.videos.push(video_file.clone());
                        }
                    }
                }
            });

            let mut video_to_delete = None;
            for (video_index, video) in self.media_library.videos.iter().enumerate() {
                let item_id = Id::new(("media_library", video_index));
                let dragged_media = DraggedMedia {
                    video_index,
                    video_file: video.clone(),
                };

                ui.dnd_drag_source(item_id, dragged_media, |ui| {
                    println!("Dragging video: {}", video.path);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("🎬");
                            ui.label(
                                std::path::Path::new(&video.path)
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy(),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.small(format!("{}x{}", video.resolution.0, video.resolution.1));
                            ui.separator();
                            ui.small(format!("{:.1}s", video.duration));
                            ui.separator();
                            ui.small(format!("{:.0}fps", video.frame_rate));
                        });
                        if !video.audio_tracks.is_empty() {
                            ui.horizontal(|ui| {
                                ui.small("🔊");
                                ui.small(format!("{} audio track(s)", video.audio_tracks.len()));
                                if let Some(first_audio) = video.audio_tracks.first() {
                                    ui.separator();
                                    ui.small(format!("{}Hz", first_audio.sample_rate));
                                    ui.separator();
                                    ui.small(format!("{} ch", first_audio.channels));
                                }
                            });
                        }

                        // Add delete button to each media item
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.small_button("🗑").clicked() {
                                video_to_delete = Some(video_index);
                            }
                        });
                    });
                });
            }

            // Handle deletion of media items
            if let Some(index) = video_to_delete {
                // Remove the video from the library
                if index < self.media_library.videos.len() {
                    self.media_library.videos.remove(index);

                    // Also remove any clips using this video from the timeline
                    for track in &mut self.timeline.tracks {
                        track.clips.retain(|clip| {
                            if let Media::Video(path) = &clip.media {
                                // Keep clips that don't match the deleted video path
                                !path.ends_with(&index.to_string())
                            } else {
                                true
                            }
                        });
                    }

                    // Reset selection if needed
                    self.timeline.selected_clip = None;
                }
            }
        });

        egui::SidePanel::right("properties").show(ctx, |ui| {
            ui.heading("Properties");

            // Show properties of the selected clip
            if let Some(clip) = self.timeline.get_selected_clip() {
                if let Media::Video(_video_path) = &clip.media {
                    ui.separator();
                    ui.label("Selected Clip:");
                    ui.small(format!("Duration: {:.2}s", clip.duration));
                    ui.small(format!("Start: {:.2}s", clip.start_time));
                }
            }
        });
    }

    fn render_playback_controls(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("playback_controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Play").clicked() {
                    self.playback.is_playing = true;
                    ctx.request_repaint();
                }
                if ui.button("Pause").clicked() {
                    self.playback.is_playing = false;
                }
                if ui.button("Stop").clicked() {
                    self.playback.is_playing = false;
                    self.timeline.current_time = 0.0;
                    ctx.request_repaint();
                }

                ui.separator();
                ui.label("Time:");
                let timeline_changed = ui
                    .add(
                        egui::Slider::new(&mut self.timeline.current_time, 0.0..=300.0).suffix("s"),
                    )
                    .changed();
                if timeline_changed {
                    ctx.request_repaint();
                }

                ui.separator();
                ui.label(format!("Speed: {:.1}x", self.playback.playback_speed));
                if ui.button("0.5x").clicked() {
                    self.playback.playback_speed = 0.5;
                }
                if ui.button("1x").clicked() {
                    self.playback.playback_speed = 1.0;
                }
                if ui.button("2x").clicked() {
                    self.playback.playback_speed = 2.0;
                }

                ui.separator();
                let (cached_frames, loading_frames) = self.frame_cache.get_cache_stats();
                ui.label(format!(
                    "Cache: {}/{} frames",
                    cached_frames - loading_frames,
                    cached_frames
                ));
            });
        });
    }

    fn is_valid_proxy_file(&self, proxy_path: &str) -> bool {
        // Check if the proxy file exists
        if !std::path::Path::new(proxy_path).exists() {
            println!("WARNING: Proxy file does not exist: {}", proxy_path);
            return false;
        }

        // Check if file is not empty
        if let Ok(metadata) = std::fs::metadata(proxy_path) {
            if metadata.len() == 0 {
                println!("WARNING: Proxy file is empty: {}", proxy_path);
                return false;
            }
        } else {
            return false;
        }

        // File exists and has content, assume it's valid
        true
    }

    // No need for custom proxy decoder since we're using standard video files now

    fn preload_upcoming_frames(&mut self) {
        let current_time = self.timeline.current_time;
        let preload_duration = 1.0; // Reduced to 1 second for better performance

        // Collect clips that need preloading to avoid borrowing issues
        let mut clips_to_preload = Vec::new();

        for track in &self.timeline.tracks {
            for clip in &track.clips {
                let clip_start = clip.start_time as f32;
                let clip_end = (clip.start_time + clip.duration) as f32;

                // Check if clip overlaps with preload window
                if current_time < clip_end && current_time + preload_duration > clip_start {
                    if let Media::Video(video_path) = &clip.media {
                        let preload_start = (current_time - clip_start).max(0.0);
                        let preload_end = (current_time + preload_duration - clip_start)
                            .min(clip.duration as f32)
                            .min(5.0); // Don't preload beyond 5 seconds to avoid seeking beyond video duration

                        if preload_end > preload_start {
                            // Store the information for later processing
                            let next_frame_time = current_time + 1.0 / 30.0; // Look ahead one frame
                            if next_frame_time >= clip_start && next_frame_time < clip_end {
                                let relative_time = next_frame_time - clip_start;
                                clips_to_preload.push((video_path.clone(), relative_time));
                            }
                        }
                    }
                }
            }
        }

        // Limit the number of frames to preload at once to prevent overwhelming the decoder
        let max_preload_frames = 3;

        // Now process the collected clips without any borrowing conflicts
        for (video_path, relative_time) in clips_to_preload.iter().take(max_preload_frames) {
            let effective_path = self.get_effective_video_path(video_path);

            // Skip preloading if the frame is already in cache
            if self
                .frame_cache
                .get_frame(&effective_path, *relative_time)
                .is_none()
            {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let path_copy = effective_path.clone();
                    let time = *relative_time;
                    use crate::video_decoder::VideoDecoder;
                    // Preloading stub: decoder removed
                    println!("Preloading frame at time {:.2}s for {}", time, path_copy);
                })) {
                    Ok(_) => {}
                    Err(e) => println!("Error during frame preloading: {:?}", e),
                }
            }
        }
    }
}

impl eframe::App for VideoEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update preview frame based on current timeline position
        self.update_preview_frame();

        // Render UI components
        self.render_menu_bar(ctx);
        self.render_side_panels(ctx);
        self.render_preview_panel(ctx);
        self.render_playback_controls(ctx);

        // Render timeline in central panel
        if let Some(drop_event) = self.timeline.render(ctx) {
            self.add_media_to_timeline(
                &drop_event.dragged_media.video_file,
                drop_event.track_index,
                drop_event.drop_time,
            );
        }

        if self.playback.is_playing {
            // Get input data outside the catch_unwind block
            let dt = ctx.input(|i| i.stable_dt);

            // Update timeline position
            self.timeline.current_time += dt * self.playback.playback_speed;

            // Only preload occasionally to avoid performance impact
            if (self.timeline.current_time * 4.0) as i32
                != ((self.timeline.current_time - dt * self.playback.playback_speed) * 8.0) as i32
                && self.timeline.current_time % 2.0 == 0.0
            {
                // Call directly instead of in catch_unwind
                self.preload_upcoming_frames();
            }

            // Calculate max time
            let max_time = self
                .timeline
                .tracks
                .iter()
                .flat_map(|track| track.clips.iter())
                .map(|clip| clip.start_time + clip.duration)
                .fold(0.0, f64::max) as f32;

            // Stop playback if the timeline reaches the end
            if max_time > 0.0 && self.timeline.current_time >= max_time {
                self.playback.is_playing = false;
                self.timeline.current_time = max_time;
            } else if self.timeline.current_time >= 300.0 {
                // Stop at 300 seconds even if no clips
                self.playback.is_playing = false;
                self.timeline.current_time = 300.0;
            }

            // Always request repaint during playback
            ctx.request_repaint();
        }
    }
}
