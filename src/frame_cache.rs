use crate::frame::Frame;
use std::collections::HashMap;
use std::time::Instant;

pub struct FrameCache {
    cache: HashMap<String, CachedFrame>,
    max_cache_size: usize,
}

#[derive(Clone)]
struct CachedFrame {
    frame: Frame,
    last_accessed: Instant,
    is_loading: bool,
}

impl FrameCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_cache_size: 100, // Cache up to 100 frames
        }
    }

    pub fn get_frame(&mut self, video_path: &str, time: f32) -> Option<Frame> {
        let cache_key = self.make_cache_key(video_path, time);

        // Check if the frame is in the cache
        if let Some(cached_frame) = self.cache.get_mut(&cache_key) {
            cached_frame.last_accessed = Instant::now();
            return Some(cached_frame.frame.clone());
        }

        // Try to find a nearby frame as a fallback
        self.find_nearby_frame(video_path, time)
    }

    pub fn preload_frames(&mut self, video_path: &str, start_time: f32, end_time: f32, fps: f32) {
        // Direct preloading is now handled by the video decoder when needed
        // This function is kept for API compatibility
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_frames = self.cache.len();
        let loading_frames = 0; // No background loading anymore
        (total_frames, loading_frames)
    }

    fn make_cache_key(&self, video_path: &str, time: f32) -> String {
        // Round time to nearest 0.1 seconds for better cache hit rate
        let rounded_time = (time * 10.0).round() / 10.0;
        format!("{}:{:.1}", video_path, rounded_time)
    }

    // This function is removed as we don't have a background decoder anymore

    fn find_nearby_frame(&self, video_path: &str, target_time: f32) -> Option<Frame> {
        let mut best_frame: Option<&CachedFrame> = None;
        let mut best_distance = f32::INFINITY;

        for (key, cached_frame) in self.cache.iter() {
            if key.starts_with(video_path) && !cached_frame.is_loading {
                if let Some(time_str) = key.split(':').nth(1) {
                    if let Ok(time) = time_str.parse::<f32>() {
                        let distance = (time - target_time).abs();
                        if distance < best_distance {
                            best_distance = distance;
                            best_frame = Some(cached_frame);
                        }
                    }
                }
            }
        }

        // Only use nearby frame if it's within 0.5 seconds
        if best_distance < 0.5 {
            return best_frame.map(|f| f.frame.clone());
        }
        None
    }

    // Add a new function to directly add a frame to the cache
    pub fn add_frame(&mut self, video_path: &str, time: f32, frame: Frame) {
        use crate::video_decoder::VideoDecoder;

        let cache_key = self.make_cache_key(video_path, time);

        // Store the frame in the cache
        self.cache.insert(
            cache_key,
            CachedFrame {
                frame,
                last_accessed: Instant::now(),
                is_loading: false,
            },
        );

        // Clean up old frames if cache is too large
        if self.cache.len() > self.max_cache_size {
            let mut entries: Vec<_> = self
                .cache
                .iter()
                .map(|(k, v)| (k.clone(), v.last_accessed))
                .collect();

            entries.sort_by_key(|(_, last_accessed)| *last_accessed);

            // Remove oldest 20% of entries
            let remove_count = self.max_cache_size / 5;
            for (key, _) in entries.into_iter().take(remove_count) {
                self.cache.remove(&key);
            }
        }
    }
}

// Drop trait no longer needed since we don't have background threads

impl Default for FrameCache {
    fn default() -> Self {
        Self::new()
    }
}
