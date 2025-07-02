#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub grid_snap: bool,
    pub playback_resolution: (u32, u32),
    pub preview_quality: PreviewQuality,
    pub auto_save: bool,
    pub auto_save_interval: u32, // seconds
    pub default_frame_rate: f32,
    pub timeline_zoom: f32,
    pub audio_enabled: bool,
    pub timeline_height: f32,
    pub track_height: f32,
    pub thumbnail_size: f32,
    pub cache_limit: usize, // number of frames to cache
    pub proxy_enabled: bool,
    pub proxy_quality: ProxyQuality,
    pub proxy_resolution: (u32, u32),
    pub auto_generate_proxies: bool,
    pub proxy_cache_max_age_hours: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewQuality {
    Low,    // 240p
    Medium, // 480p
    High,   // 720p
    Ultra,  // 1080p
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyQuality {
    Draft,   // Very low quality, fast generation
    Preview, // Medium quality, good for editing
    High,    // High quality, slower generation
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            grid_snap: false,
            playback_resolution: (1920, 1080),
            preview_quality: PreviewQuality::Medium,
            auto_save: true,
            auto_save_interval: 300, // 5 minutes
            default_frame_rate: 30.0,
            timeline_zoom: 1.0,
            audio_enabled: true,
            timeline_height: 200.0,
            track_height: 40.0,
            thumbnail_size: 16.0,
            cache_limit: 50,
            proxy_enabled: true,
            proxy_quality: ProxyQuality::Preview,
            proxy_resolution: (480, 270),
            auto_generate_proxies: true,
            proxy_cache_max_age_hours: 24,
        }
    }
}

impl EditorSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_preview_resolution(&self) -> (u32, u32) {
        match self.preview_quality {
            PreviewQuality::Low => (320, 240),
            PreviewQuality::Medium => (640, 480),
            PreviewQuality::High => (1280, 720),
            PreviewQuality::Ultra => (1920, 1080),
        }
    }

    pub fn set_preview_quality(&mut self, quality: PreviewQuality) {
        self.preview_quality = quality;
    }

    pub fn toggle_grid_snap(&mut self) {
        self.grid_snap = !self.grid_snap;
    }

    pub fn set_timeline_zoom(&mut self, zoom: f32) {
        self.timeline_zoom = zoom.max(0.1).min(10.0);
    }

    pub fn zoom_in(&mut self) {
        self.set_timeline_zoom(self.timeline_zoom * 1.2);
    }

    pub fn zoom_out(&mut self) {
        self.set_timeline_zoom(self.timeline_zoom / 1.2);
    }

    pub fn reset_zoom(&mut self) {
        self.timeline_zoom = 1.0;
    }

    pub fn set_cache_limit(&mut self, limit: usize) {
        self.cache_limit = limit.max(10).min(1000);
    }

    pub fn set_proxy_quality(&mut self, quality: ProxyQuality) {
        self.proxy_quality = quality;
    }

    pub fn toggle_proxy_enabled(&mut self) {
        self.proxy_enabled = !self.proxy_enabled;
    }

    pub fn set_proxy_resolution(&mut self, resolution: (u32, u32)) {
        self.proxy_resolution = resolution;
    }

    pub fn get_proxy_settings(&self) -> crate::proxy_generator::ProxySettings {
        crate::proxy_generator::ProxySettings {
            resolution: self.proxy_resolution,
            frame_rate: self.default_frame_rate,
            quality: match self.proxy_quality {
                ProxyQuality::Draft => crate::proxy_generator::ProxyQuality::Draft,
                ProxyQuality::Preview => crate::proxy_generator::ProxyQuality::Preview,
                ProxyQuality::High => crate::proxy_generator::ProxyQuality::High,
            },
        }
    }

    pub fn validate(&mut self) {
        // Ensure all values are within reasonable bounds
        self.default_frame_rate = self.default_frame_rate.max(1.0).min(120.0);
        self.auto_save_interval = self.auto_save_interval.max(60).min(3600);
        self.timeline_height = self.timeline_height.max(100.0).min(500.0);
        self.track_height = self.track_height.max(20.0).min(100.0);
        self.thumbnail_size = self.thumbnail_size.max(8.0).min(64.0);
        self.timeline_zoom = self.timeline_zoom.max(0.1).min(10.0);
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Implement loading from file (JSON/TOML)
        // For now, return default settings
        Ok(Self::default())
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement saving to file (JSON/TOML)
        // For now, just return Ok
        Ok(())
    }
}
