pub struct PlaybackState {
    pub is_playing: bool,
    pub playback_speed: f32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            playback_speed: 1.0,
        }
    }
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            playback_speed: 1.0,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1).min(4.0);
    }
}
