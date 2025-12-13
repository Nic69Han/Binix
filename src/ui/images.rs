//! Image loading and caching for the browser

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use egui::ColorImage;

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

/// Image cache for loaded images
#[derive(Default)]
pub struct ImageCache {
    /// Cached images by URL
    pub images: HashMap<String, ImageState>,
    /// Pending load requests
    pending: Vec<String>,
}

impl ImageCache {
    /// Create a new image cache
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            pending: Vec::new(),
        }
    }

    /// Get image state for a URL
    pub fn get(&self, url: &str) -> Option<&ImageState> {
        self.images.get(url)
    }

    /// Request an image to be loaded
    pub fn request(&mut self, url: &str) {
        if !self.images.contains_key(url) {
            self.images.insert(url.to_string(), ImageState::Loading);
            self.pending.push(url.to_string());
        }
    }

    /// Mark an image as loaded
    pub fn set_loaded(&mut self, url: &str, image: LoadedImage) {
        self.images.insert(url.to_string(), ImageState::Loaded(image));
    }

    /// Mark an image as failed
    pub fn set_failed(&mut self, url: &str, error: String) {
        self.images.insert(url.to_string(), ImageState::Failed(error));
    }

    /// Get pending URLs to load
    pub fn take_pending(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending)
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

