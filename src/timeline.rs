use crate::media::{Media, VideoFile};
use eframe::egui::{self, CornerRadius};

pub struct TimeLine {
    pub tracks: Vec<Track>,
    pub current_time: f32,
    pub selected_clip: Option<(usize, usize)>, // (track_index, clip_index)
}

impl Default for TimeLine {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            current_time: 0.0,
            selected_clip: None,
        }
    }
}

pub struct Track {
    pub kind: TrackKind,
    pub clips: Vec<Clip>,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            kind: TrackKind::Video,
            clips: Vec::new(),
        }
    }
}

#[derive(Default)]
pub enum TrackKind {
    #[default]
    Video,
    Audio,
    Effect,
}

pub struct Clip {
    pub start_time: f64,
    pub duration: f64,
    pub media: Media,
    pub is_being_dragged: bool,
    pub drag_offset: f64,
}

#[derive(Debug)]

pub struct TimelineDropEvent {
    pub dragged_media: DraggedMedia,
    pub track_index: usize,
    pub drop_time: f64,
}

#[derive(Clone, Debug)]
pub struct DraggedMedia {
    pub video_index: usize,
    pub video_file: VideoFile,
}

#[derive(Debug)]
pub struct DraggedClip {
    pub track_index: usize,
    pub clip_index: usize,
    pub original_start_time: f64,
}

impl TimeLine {
    pub fn render(&mut self, ctx: &egui::Context) -> Option<TimelineDropEvent> {
        let mut drop_event = None;
        // Store drag operations to apply after the iteration
        let mut drag_operations: Vec<(usize, usize, bool, Option<f64>)> = Vec::new();
        // Store clip deletion operations
        let mut deletion_operations: Vec<(usize, usize)> = Vec::new();

        egui::TopBottomPanel::top("Timeline").show(ctx, |ui| {
            ui.heading("Timeline");
            ui.separator();

            // Add controls
            ui.horizontal(|ui| {
                if ui.button("+ Video Track").clicked() {
                    self.tracks.push(Track {
                        kind: TrackKind::Video,
                        clips: Vec::new(),
                    });
                }
                if ui.button("+ Audio Track").clicked() {
                    self.tracks.push(Track {
                        kind: TrackKind::Audio,
                        clips: Vec::new(),
                    });
                }
                if ui.button("Delete Selected").clicked() {
                    self.delete_selected_clip();
                }
                ui.separator();
                ui.label(format!("Selected: {:?}", self.selected_clip));
            });
            ui.separator();

            // Show timeline tracks
            for (track_index, track) in self.tracks.iter().enumerate() {
                ui.horizontal(|ui| {
                    let track_type = match track.kind {
                        TrackKind::Video => "Video",
                        TrackKind::Audio => "Audio",
                        TrackKind::Effect => "Effect",
                    };
                    ui.label(format!("{} Track {}", track_type, track_index + 1));

                    // Create a canvas for clips with drop zone functionality
                    let available_width = ui.available_width() - 100.0; // Reserve space for track label

                    let frame = egui::Frame::default().inner_margin(0.0);
                    let (_, dropped_payload) = ui.dnd_drop_zone::<DraggedMedia, ()>(frame, |ui| {
                        let (response, painter) = ui.allocate_painter(
                            egui::vec2(available_width, 40.0),
                            egui::Sense::click(),
                        );

                        // Draw track background
                        painter.rect_filled(
                            response.rect,
                            CornerRadius::same(2),
                            egui::Color32::from_gray(40),
                        );

                        // Show clips in track
                        for (clip_index, clip) in track.clips.iter().enumerate() {
                            let start_x = (clip.start_time as f32 / 30.0) * available_width; // Scale to 30 seconds max
                            let width = (clip.duration as f32 / 30.0) * available_width;

                            let clip_rect = egui::Rect::from_min_size(
                                response.rect.min + egui::vec2(start_x, 5.0),
                                egui::vec2(width.max(80.0), 30.0), // Minimum width for thumbnails
                            );

                            // Check if this clip is selected
                            let is_selected = self.selected_clip == Some((track_index, clip_index));

                            // Draw clip background based on track type and selection
                            let clip_color = match (&track.kind, is_selected) {
                                (TrackKind::Video, true) => egui::Color32::from_rgb(120, 180, 240),
                                (TrackKind::Video, false) => egui::Color32::from_rgb(70, 130, 180),
                                (TrackKind::Audio, true) => egui::Color32::from_rgb(120, 240, 120),
                                (TrackKind::Audio, false) => egui::Color32::from_rgb(70, 180, 70),
                                (TrackKind::Effect, true) => egui::Color32::from_rgb(240, 120, 120),
                                (TrackKind::Effect, false) => egui::Color32::from_rgb(180, 70, 70),
                            };

                            painter.rect_filled(clip_rect, CornerRadius::same(3), clip_color);

                            // Handle clip interactions
                            let clip_response =
                                ui.allocate_rect(clip_rect, egui::Sense::click_and_drag());
                            if clip_response.clicked() {
                                self.selected_clip = Some((track_index, clip_index));
                            }
                            if clip_response.secondary_clicked() {
                                // Right-click context menu
                                deletion_operations.push((track_index, clip_index));
                            }

                            // Handle clip dragging
                            if clip_response.drag_started() {
                                // Store drag start info for later
                                let mut drag_offset = 0.0;
                                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                                    let relative_x = pointer_pos.x - clip_rect.min.x;
                                    drag_offset =
                                        (relative_x / clip_rect.width()) as f64 * clip.duration;
                                }
                                // Add to operations: (track_index, clip_index, is_drag_start, Some(offset))
                                drag_operations.push((
                                    track_index,
                                    clip_index,
                                    true,
                                    Some(drag_offset),
                                ));
                            }

                            if clip_response.dragged() && clip.is_being_dragged {
                                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                                    let relative_x = pointer_pos.x - response.rect.min.x;
                                    let new_start =
                                        ((relative_x / available_width) * 30.0).max(0.0) as f64;

                                    // Add to operations: (track_index, clip_index, is_dragging, Some(new_position))
                                    drag_operations.push((
                                        track_index,
                                        clip_index,
                                        false,
                                        Some(new_start),
                                    ));
                                }
                                ui.ctx().request_repaint(); // Ensure smooth updates
                            }

                            if clip_response.drag_stopped() {
                                // Add to operations: (track_index, clip_index, is_drag_stop, None)
                                drag_operations.push((track_index, clip_index, false, None));
                            }

                            // Draw waveform for audio clips or thumbnail placeholder for video
                            match (&clip.media, &track.kind) {
                                (Media::Audio(_), TrackKind::Audio) => {
                                    // Draw simple waveform representation
                                    let wave_height = clip_rect.height() * 0.4;
                                    let wave_y = clip_rect.center().y;
                                    for x in (clip_rect.left() as i32..clip_rect.right() as i32)
                                        .step_by(4)
                                    {
                                        let wave_amplitude =
                                            ((x as f32 * 0.1).sin() * wave_height * 0.5).abs();
                                        painter.line_segment(
                                            [
                                                egui::pos2(x as f32, wave_y - wave_amplitude),
                                                egui::pos2(x as f32, wave_y + wave_amplitude),
                                            ],
                                            egui::Stroke::new(1.0, egui::Color32::WHITE),
                                        );
                                    }
                                }
                                _ => {
                                    // Keep existing behavior for video clips
                                }
                            }

                            // Draw clip border
                            painter.rect_stroke(
                                clip_rect,
                                CornerRadius::same(3),
                                egui::Stroke::new(1.0, egui::Color32::WHITE),
                                egui::StrokeKind::Inside,
                            );

                            // Draw clip filename at bottom
                            if let Media::Video(video_path) = &clip.media {
                                let filename = std::path::Path::new(video_path)
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy();

                                painter.text(
                                    egui::pos2(clip_rect.left() + 2.0, clip_rect.bottom() - 12.0),
                                    egui::Align2::LEFT_BOTTOM,
                                    filename,
                                    egui::FontId::proportional(8.0),
                                    egui::Color32::WHITE,
                                );
                            }
                        }

                        // Draw playback cursor
                        let cursor_x = (self.current_time / 30.0) * available_width;
                        if cursor_x <= available_width {
                            painter.line_segment(
                                [
                                    response.rect.min + egui::vec2(cursor_x, 0.0),
                                    response.rect.min + egui::vec2(cursor_x, 40.0),
                                ],
                                egui::Stroke::new(2.0, egui::Color32::RED),
                            );
                        }

                        // Handle hover feedback for drag and drop
                        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                            if let Some(_hovered_payload) =
                                response.dnd_hover_payload::<DraggedMedia>()
                            {
                                if response.rect.contains(pointer_pos) {
                                    // Calculate drop position based on mouse x position
                                    let relative_x = pointer_pos.x - response.rect.min.x;
                                    let _drop_time = (relative_x / available_width) * 30.0;

                                    // Draw drop indicator line
                                    painter.line_segment(
                                        [
                                            egui::pos2(pointer_pos.x, response.rect.min.y),
                                            egui::pos2(pointer_pos.x, response.rect.max.y),
                                        ],
                                        egui::Stroke::new(2.0, egui::Color32::GREEN),
                                    );
                                }
                            }
                        }
                    });

                    // Handle dropped media
                    if let Some(dragged_media) = dropped_payload {
                        // Calculate drop position based on mouse position
                        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                            let relative_x = pointer_pos.x - (ui.min_rect().min.x + 100.0);
                            let drop_time = ((relative_x / available_width) * 30.0).max(0.0) as f64;

                            drop_event = Some(TimelineDropEvent {
                                dragged_media: (*dragged_media).clone(),
                                track_index,
                                drop_time,
                            });
                        }
                    }
                });
                ui.add_space(5.0);
            }
        });

        // Apply drag operations
        for (track_idx, clip_idx, is_drag_start, maybe_pos) in drag_operations {
            if track_idx < self.tracks.len() && clip_idx < self.tracks[track_idx].clips.len() {
                let clip = &mut self.tracks[track_idx].clips[clip_idx];

                if is_drag_start {
                    clip.is_being_dragged = true;
                    if let Some(offset) = maybe_pos {
                        clip.drag_offset = offset;
                    }
                } else if let Some(new_pos) = maybe_pos {
                    // It's a drag movement
                    clip.start_time = (new_pos - clip.drag_offset).max(0.0);
                } else {
                    // It's a drag stop
                    clip.is_being_dragged = false;
                }
            }
        }

        // Apply deletion operations - in reverse order to avoid index issues
        deletion_operations.sort_by(|a, b| b.1.cmp(&a.1));
        for (track_idx, clip_idx) in deletion_operations {
            if track_idx < self.tracks.len() && clip_idx < self.tracks[track_idx].clips.len() {
                self.tracks[track_idx].clips.remove(clip_idx);

                // Update selection if needed
                if self.selected_clip == Some((track_idx, clip_idx)) {
                    self.selected_clip = None;
                } else if let Some((sel_track, sel_clip)) = self.selected_clip {
                    if sel_track == track_idx && sel_clip > clip_idx {
                        self.selected_clip = Some((sel_track, sel_clip - 1));
                    }
                }
            }
        }

        drop_event
    }

    pub fn delete_selected_clip(&mut self) {
        if let Some((track_index, clip_index)) = self.selected_clip {
            if track_index < self.tracks.len() && clip_index < self.tracks[track_index].clips.len()
            {
                self.tracks[track_index].clips.remove(clip_index);
                self.selected_clip = None;
            }
        }
    }

    pub fn get_selected_clip(&self) -> Option<&crate::timeline::Clip> {
        if let Some((track_index, clip_index)) = self.selected_clip {
            self.tracks.get(track_index)?.clips.get(clip_index)
        } else {
            None
        }
    }
}
