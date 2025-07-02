use crate::frame::Frame;
use crate::video_decoder::VideoDecoder;
use std::collections::HashMap;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct ProxyGenerator {
    proxy_cache_dir: PathBuf,
    generation_queue: Arc<Mutex<Vec<ProxyRequest>>>,
    proxy_registry: Arc<Mutex<HashMap<String, ProxyInfo>>>,
    worker_running: Arc<Mutex<bool>>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Clone)]
struct ProxyRequest {
    video_path: String,
    proxy_settings: ProxySettings,
    priority: i32,
}

#[derive(Clone, PartialEq)]
pub struct ProxySettings {
    pub resolution: (u32, u32),
    pub frame_rate: f32,
    pub quality: ProxyQuality,
}

#[derive(Clone, PartialEq)]
pub enum ProxyQuality {
    Draft,   // Very low quality, fast generation
    Preview, // Medium quality, good for editing
    High,    // High quality, slower generation
}

#[derive(Clone, PartialEq)]
pub struct ProxyInfo {
    pub proxy_path: String,
    pub original_path: String,
    pub settings: ProxySettings,
    pub generation_progress: f32,
    pub is_ready: bool,
    pub file_size: u64,
    pub created_at: Instant,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            resolution: (480, 270), // Quarter HD for good performance
            frame_rate: 30.0,
            quality: ProxyQuality::Preview,
        }
    }
}

impl ProxyGenerator {
    pub fn new() -> std::io::Result<Self> {
        let proxy_cache_dir = std::env::temp_dir().join("dino_proxy_cache");
        std::fs::create_dir_all(&proxy_cache_dir)?;

        let generation_queue = Arc::new(Mutex::new(Vec::new()));
        let proxy_registry = Arc::new(Mutex::new(HashMap::new()));
        let worker_running = Arc::new(Mutex::new(true));

        let worker_handle = Some(Self::start_worker_thread(
            proxy_cache_dir.clone(),
            generation_queue.clone(),
            proxy_registry.clone(),
            worker_running.clone(),
        ));

        Ok(Self {
            proxy_cache_dir,
            generation_queue,
            proxy_registry,
            worker_running,
            worker_handle,
        })
    }

    pub fn request_proxy(&self, video_path: &str, settings: ProxySettings, priority: i32) {
        let proxy_id = self.generate_proxy_id(video_path, &settings);

        // Check if proxy already exists or is being generated
        if let Ok(registry) = self.proxy_registry.lock() {
            if registry.contains_key(&proxy_id) {
                return; // Already exists or in progress
            }
        }

        // Add to generation queue
        let request = ProxyRequest {
            video_path: video_path.to_string(),
            proxy_settings: settings.clone(),
            priority,
        };

        if let Ok(mut queue) = self.generation_queue.lock() {
            queue.push(request);
            queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        // Register as pending
        if let Ok(mut registry) = self.proxy_registry.lock() {
            registry.insert(
                proxy_id,
                ProxyInfo {
                    proxy_path: self.generate_proxy_path(video_path, &settings),
                    original_path: video_path.to_string(),
                    settings,
                    generation_progress: 0.0,
                    is_ready: false,
                    file_size: 0,
                    created_at: Instant::now(),
                },
            );
        }
    }

    pub fn get_proxy_info(&self, video_path: &str, settings: &ProxySettings) -> Option<ProxyInfo> {
        let proxy_id = self.generate_proxy_id(video_path, settings);
        if let Ok(registry) = self.proxy_registry.lock() {
            registry.get(&proxy_id).cloned()
        } else {
            None
        }
    }

    pub fn is_proxy_ready(&self, video_path: &str, settings: &ProxySettings) -> bool {
        self.get_proxy_info(video_path, settings)
            .map(|info| info.is_ready)
            .unwrap_or(false)
    }

    pub fn get_proxy_path(&self, video_path: &str, settings: &ProxySettings) -> Option<String> {
        if self.is_proxy_ready(video_path, settings) {
            let proxy_path = self.generate_proxy_path(video_path, settings);
            // Verify that the proxy file actually exists
            if std::path::Path::new(&proxy_path).exists() {
                Some(proxy_path)
            } else {
                println!(
                    "Warning: Proxy marked as ready but file not found: {}",
                    proxy_path
                );
                None
            }
        } else {
            None
        }
    }

    pub fn cleanup_old_proxies(&self, max_age_hours: u64) {
        let cutoff_time = Instant::now() - std::time::Duration::from_secs(max_age_hours * 3600);

        if let Ok(mut registry) = self.proxy_registry.lock() {
            let mut to_remove = Vec::new();

            for (proxy_id, info) in registry.iter() {
                if info.created_at < cutoff_time {
                    // Remove file
                    let _ = std::fs::remove_file(&info.proxy_path);
                    to_remove.push(proxy_id.clone());
                }
            }

            for proxy_id in to_remove {
                registry.remove(&proxy_id);
            }
        }
    }

    pub fn get_cache_stats(&self) -> (usize, usize, u64) {
        if let Ok(registry) = self.proxy_registry.lock() {
            let total_proxies = registry.len();
            let ready_proxies = registry.values().filter(|info| info.is_ready).count();
            let total_size = registry.values().map(|info| info.file_size).sum();
            (total_proxies, ready_proxies, total_size)
        } else {
            (0, 0, 0)
        }
    }

    fn generate_proxy_id(&self, video_path: &str, settings: &ProxySettings) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        video_path.hash(&mut hasher);
        settings.resolution.hash(&mut hasher);
        (settings.frame_rate as u32).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn generate_proxy_path(&self, video_path: &str, settings: &ProxySettings) -> String {
        let proxy_id = self.generate_proxy_id(video_path, settings);
        let filename = format!("proxy_{}_{}.mp4", proxy_id, settings.frame_rate as u32);
        self.proxy_cache_dir
            .join(filename)
            .to_string_lossy()
            .to_string()
    }

    fn start_worker_thread(
        _cache_dir: PathBuf,
        queue: Arc<Mutex<Vec<ProxyRequest>>>,
        registry: Arc<Mutex<HashMap<String, ProxyInfo>>>,
        running: Arc<Mutex<bool>>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            while {
                if let Ok(is_running) = running.lock() {
                    *is_running
                } else {
                    false
                }
            } {
                // Get next request
                let request = if let Ok(mut q) = queue.lock() {
                    q.pop()
                } else {
                    None
                };

                if let Some(req) = request {
                    Self::generate_proxy_file(req, &registry);
                } else {
                    // No work to do, sleep briefly
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        })
    }

    fn generate_proxy_file(
        request: ProxyRequest,
        registry: &Arc<Mutex<HashMap<String, ProxyInfo>>>,
    ) {
        let proxy_id = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            request.video_path.hash(&mut hasher);
            request.proxy_settings.resolution.hash(&mut hasher);
            (request.proxy_settings.frame_rate as u32).hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };

        println!("Generating proxy for: {}", request.video_path);

        // Check if FFmpeg is available
        if !Self::is_ffmpeg_available() {
            println!("FFmpeg not found. Cannot generate proxy video.");
            return;
        }

        // Get the proxy path
        let proxy_path = {
            let mut proxy_path = String::new();
            if let Ok(reg) = registry.lock() {
                if let Some(info) = reg.get(&proxy_id) {
                    proxy_path = info.proxy_path.clone();
                }
            }
            proxy_path
        };

        // Create parent directory if needed
        let proxy_dir = std::path::Path::new(&proxy_path).parent();
        if let Some(dir) = proxy_dir {
            if !dir.exists() {
                let _ = std::fs::create_dir_all(dir);
            }
        }

        // Update progress to indicate we're starting
        if let Ok(mut reg) = registry.lock() {
            if let Some(info) = reg.get_mut(&proxy_id) {
                info.generation_progress = 0.1;
            }
        }

        // Get video parameters from request
        let width = request.proxy_settings.resolution.0;
        let height = request.proxy_settings.resolution.1;
        let frame_rate = request.proxy_settings.frame_rate;

        // Determine quality settings based on proxy quality
        let crf = match request.proxy_settings.quality {
            ProxyQuality::Draft => "28",   // Lower quality, smaller file
            ProxyQuality::Preview => "23", // Medium quality
            ProxyQuality::High => "18",    // Higher quality
        };

        // Use FFmpeg to generate the proxy file
        let result = Self::generate_proxy_with_ffmpeg(
            &request.video_path,
            &proxy_path,
            width,
            height,
            frame_rate,
            crf,
            registry,
            &proxy_id,
        );

        // Get file size of the generated proxy file
        let file_size = if result && std::path::Path::new(&proxy_path).exists() {
            if let Ok(metadata) = std::fs::metadata(&proxy_path) {
                metadata.len()
            } else {
                0 // File doesn't exist or can't be read
            }
        } else {
            0 // Generation failed
        };

        // Update status in registry
        if let Ok(mut reg) = registry.lock() {
            if let Some(info) = reg.get_mut(&proxy_id) {
                info.is_ready = file_size > 0;
                info.generation_progress = if file_size > 0 { 1.0 } else { 0.0 };
                info.file_size = file_size;
            }
        }

        if file_size > 0 {
            println!(
                "Proxy generation completed for: {} (saved to {}, size: {}KB)",
                request.video_path,
                proxy_path,
                file_size / 1024
            );
        } else {
            println!("Proxy generation failed for: {}", request.video_path);
        }
    }

    // Check if FFmpeg is available on the system
    fn is_ffmpeg_available() -> bool {
        use std::process::Command;

        let output = Command::new("ffmpeg").arg("-version").output();

        if let Ok(output) = output {
            return output.status.success();
        }

        println!("FFmpeg not found. Please install FFmpeg to enable proxy generation.");
        false
    }

    // Generate proxy video using FFmpeg as an external process
    fn generate_proxy_with_ffmpeg(
        input_path: &str,
        output_path: &str,
        width: u32,
        height: u32,
        frame_rate: f32,
        crf: &str,
        registry: &Arc<Mutex<HashMap<String, ProxyInfo>>>,
        proxy_id: &str,
    ) -> bool {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        use std::thread;
        use std::time::Duration;

        println!(
            "Starting FFmpeg proxy generation: {} -> {}",
            input_path, output_path
        );

        // Build FFmpeg command
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y") // Overwrite output files without asking
            .arg("-i")
            .arg(input_path)
            .arg("-vf")
            .arg(format!("scale={}:{}", width, height))
            .arg("-r")
            .arg(frame_rate.to_string())
            .arg("-c:v")
            .arg("libx264")
            .arg("-crf")
            .arg(crf)
            .arg("-preset")
            .arg("veryfast") // Fast encoding
            .arg("-an") // No audio
            .arg("-movflags")
            .arg("faststart") // Optimize for streaming
            .arg(output_path)
            .stderr(Stdio::piped()); // Capture stderr to parse progress

        // Start the FFmpeg process
        let mut process = match cmd.spawn() {
            Ok(p) => p,
            Err(e) => {
                println!("Failed to start FFmpeg: {}", e);
                return false;
            }
        };

        // Get the stderr to parse progress
        if let Some(stderr) = process.stderr.take() {
            let reader = BufReader::new(stderr);
            let proxy_id_clone = proxy_id.to_string();
            let registry_clone = registry.clone();

            // Start a thread to parse progress
            thread::spawn(move || {
                let mut duration_secs = 0.0;
                let mut time_secs = 0.0;

                for line in reader.lines() {
                    if let Ok(line) = line {
                        // Try to extract duration
                        if line.contains("Duration:") && duration_secs == 0.0 {
                            if let Some(time_str) = Self::extract_time_string(&line) {
                                duration_secs = Self::parse_time_string(time_str);
                            }
                        }

                        // Try to extract current time
                        if line.contains("time=") && duration_secs > 0.0 {
                            if let Some(time_str) = Self::extract_time_string(&line) {
                                time_secs = Self::parse_time_string(time_str);

                                // Update progress
                                if let Ok(mut reg) = registry_clone.lock() {
                                    if let Some(info) = reg.get_mut(&proxy_id_clone) {
                                        let progress = if duration_secs > 0.0 {
                                            (time_secs / duration_secs).min(0.99)
                                        } else {
                                            0.5 // If we can't determine duration
                                        };
                                        info.generation_progress = progress;
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        // Wait for the process to complete
        match process.wait() {
            Ok(status) => {
                if status.success() {
                    println!("FFmpeg completed successfully for {}", output_path);
                    true
                } else {
                    println!("FFmpeg failed with status: {}", status);
                    false
                }
            }
            Err(e) => {
                println!("Failed to wait for FFmpeg: {}", e);
                false
            }
        }
    }

    // Extract time string from FFmpeg output
    fn extract_time_string(line: &str) -> Option<&str> {
        // For "Duration: 00:01:23.45" format
        if line.contains("Duration:") {
            let start = line.find("Duration:")? + 9;
            let end = line[start..].find(",")?;
            return Some(line[start..start + end].trim());
        }

        // For "time=00:01:23.45" format
        if line.contains("time=") {
            let start = line.find("time=")? + 5;
            let end = line[start..].find(" ")?;
            return Some(line[start..start + end].trim());
        }

        None
    }

    // Parse time string in format "00:01:23.45" to seconds
    fn parse_time_string(time_str: &str) -> f32 {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return 0.0;
        }

        let hours: f32 = parts[0].parse().unwrap_or(0.0);
        let minutes: f32 = parts[1].parse().unwrap_or(0.0);
        let seconds: f32 = parts[2].parse().unwrap_or(0.0);

        hours * 3600.0 + minutes * 60.0 + seconds
    }
}

impl Drop for ProxyGenerator {
    fn drop(&mut self) {
        // Signal worker to stop
        if let Ok(mut running) = self.worker_running.lock() {
            *running = false;
        }

        // Wait for worker to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Default for ProxyGenerator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback if we can't create the cache directory
            Self {
                proxy_cache_dir: PathBuf::new(),
                generation_queue: Arc::new(Mutex::new(Vec::new())),
                proxy_registry: Arc::new(Mutex::new(HashMap::new())),
                worker_running: Arc::new(Mutex::new(false)),
                worker_handle: None,
            }
        })
    }
}
