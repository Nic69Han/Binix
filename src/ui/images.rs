//! Image loading and caching for the browser
//!
//! Implements lazy loading with intersection observer pattern.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use egui::ColorImage;

/// Image loading priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    /// Critical - above the fold
    Critical = 0,
    /// High - visible soon
    High = 1,
    /// Normal - regular images
    Normal = 2,
    /// Low - lazy loaded
    Low = 3,
    /// Idle - load when nothing else to do
    Idle = 4,
}

impl Default for LoadPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Loaded image data
#[derive(Clone)]
pub struct LoadedImage {
    /// Image data
    pub data: Arc<ColorImage>,
    /// Original URL
    pub url: String,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

/// Image loading state
#[derive(Clone)]
pub enum ImageState {
    /// Not yet started loading
    NotLoaded,
    /// Currently loading
    Loading,
    /// Successfully loaded
    Loaded(LoadedImage),
    /// Failed to load
    Failed(String),
}

/// Pending image request with priority
#[derive(Clone)]
pub struct PendingImage {
    /// Image URL
    pub url: String,
    /// Load priority
    pub priority: LoadPriority,
    /// Position in viewport (distance from top)
    pub viewport_distance: f32,
}

/// Image cache for loaded images with lazy loading
#[derive(Default)]
pub struct ImageCache {
    /// Cached images by URL
    pub images: HashMap<String, ImageState>,
    /// Pending load requests with priority
    pending: Vec<PendingImage>,
    /// Viewport height for calculating visibility
    viewport_height: f32,
    /// Current scroll position
    scroll_position: f32,
}

impl ImageCache {
    /// Create a new image cache
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            pending: Vec::new(),
            viewport_height: 800.0,
            scroll_position: 0.0,
        }
    }

    /// Update viewport info for lazy loading
    pub fn update_viewport(&mut self, height: f32, scroll_y: f32) {
        self.viewport_height = height;
        self.scroll_position = scroll_y;
    }

    /// Check if a position is in or near the viewport
    pub fn is_in_viewport(&self, y_position: f32, margin: f32) -> bool {
        let viewport_top = self.scroll_position - margin;
        let viewport_bottom = self.scroll_position + self.viewport_height + margin;
        y_position >= viewport_top && y_position <= viewport_bottom
    }

    /// Get image state for a URL
    pub fn get(&self, url: &str) -> Option<&ImageState> {
        self.images.get(url)
    }

    /// Request an image to be loaded (legacy API)
    pub fn request(&mut self, url: &str) {
        self.request_with_priority(url, LoadPriority::Normal, 0.0);
    }

    /// Request an image with priority based on position
    pub fn request_with_priority(&mut self, url: &str, priority: LoadPriority, y_position: f32) {
        if !self.images.contains_key(url) {
            self.images.insert(url.to_string(), ImageState::Loading);
            self.pending.push(PendingImage {
                url: url.to_string(),
                priority,
                viewport_distance: (y_position - self.scroll_position).abs(),
            });
        }
    }

    /// Request lazy loading (only if in viewport or near)
    pub fn request_lazy(&mut self, url: &str, y_position: f32) -> bool {
        // Load immediately if in viewport
        if self.is_in_viewport(y_position, self.viewport_height) {
            self.request_with_priority(url, LoadPriority::High, y_position);
            return true;
        }

        // Mark as not loaded but don't fetch yet
        if !self.images.contains_key(url) {
            self.images.insert(url.to_string(), ImageState::NotLoaded);
        }
        false
    }

    /// Mark an image as loaded
    pub fn set_loaded(&mut self, url: &str, image: LoadedImage) {
        self.images.insert(url.to_string(), ImageState::Loaded(image));
    }

    /// Mark an image as failed
    pub fn set_failed(&mut self, url: &str, error: String) {
        self.images.insert(url.to_string(), ImageState::Failed(error));
    }

    /// Get pending URLs to load, sorted by priority
    pub fn take_pending(&mut self) -> Vec<String> {
        // Sort by priority, then by viewport distance
        self.pending.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then(a.viewport_distance.partial_cmp(&b.viewport_distance).unwrap_or(std::cmp::Ordering::Equal))
        });

        self.pending.drain(..).map(|p| p.url).collect()
    }

    /// Get limited pending URLs (for bandwidth management)
    pub fn take_pending_limited(&mut self, max_concurrent: usize) -> Vec<String> {
        self.pending.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then(a.viewport_distance.partial_cmp(&b.viewport_distance).unwrap_or(std::cmp::Ordering::Equal))
        });

        let to_load: Vec<_> = self.pending.drain(..max_concurrent.min(self.pending.len()))
            .map(|p| p.url)
            .collect();

        to_load
    }

    /// Check for lazy images that should now load (after scroll)
    pub fn check_lazy_images(&mut self) {
        let urls_to_load: Vec<String> = self.images.iter()
            .filter_map(|(url, state)| {
                if matches!(state, ImageState::NotLoaded) {
                    Some(url.clone())
                } else {
                    None
                }
            })
            .collect();

        for url in urls_to_load {
            // Re-check viewport and potentially load
            // Note: This would need y_position stored, simplified here
            self.images.insert(url.clone(), ImageState::Loading);
            self.pending.push(PendingImage {
                url,
                priority: LoadPriority::Normal,
                viewport_distance: 0.0,
            });
        }
    }
}

/// Global shared image cache
pub type SharedImageCache = Arc<Mutex<ImageCache>>;

/// Create a shared image cache
pub fn create_shared_cache() -> SharedImageCache {
    Arc::new(Mutex::new(ImageCache::new()))
}

/// Load an image from bytes
pub fn decode_image(bytes: &[u8]) -> Result<ColorImage, String> {
    // Try to load with image crate
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    
    Ok(ColorImage::from_rgba_unmultiplied(size, &pixels))
}

/// Create a placeholder image for loading state
pub fn placeholder_image(width: u32, height: u32) -> ColorImage {
    let size = [width as usize, height as usize];
    let mut pixels = vec![200u8; size[0] * size[1] * 4]; // Gray
    
    // Add alpha channel
    for i in (0..pixels.len()).step_by(4) {
        pixels[i] = 200;     // R
        pixels[i + 1] = 200; // G
        pixels[i + 2] = 200; // B
        pixels[i + 3] = 255; // A
    }
    
    ColorImage::from_rgba_unmultiplied(size, &pixels)
}

/// Create an error placeholder image
pub fn error_image(width: u32, height: u32) -> ColorImage {
    let size = [width as usize, height as usize];
    let mut pixels = vec![0u8; size[0] * size[1] * 4];
    
    // Red-ish error color
    for i in (0..pixels.len()).step_by(4) {
        pixels[i] = 255;     // R
        pixels[i + 1] = 200; // G
        pixels[i + 2] = 200; // B
        pixels[i + 3] = 255; // A
    }
    
    ColorImage::from_rgba_unmultiplied(size, &pixels)
}

