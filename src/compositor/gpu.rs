//! GPU backend using wgpu for hardware-accelerated rendering

use wgpu::{
    Adapter, Device, Instance, Queue,
    TextureFormat, TextureUsages, TextureView,
    Buffer, BufferUsages, RequestDeviceError,
};

/// GPU rendering context
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Option<Adapter>,
    pub device: Option<Device>,
    pub queue: Option<Queue>,
    pub surface_format: TextureFormat,
}

impl GpuContext {
    /// Create a new GPU context
    pub fn new() -> Self {
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        Self {
            instance,
            adapter: None,
            device: None,
            queue: None,
            surface_format: TextureFormat::Bgra8UnormSrgb,
        }
    }

    /// Initialize the GPU context (async)
    pub async fn initialize(&mut self) -> Result<(), GpuError> {
        // Request adapter
        let adapter = self.instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| GpuError::NoAdapter)?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Binix GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::default(),
            })
            .await
            .map_err(|e: RequestDeviceError| GpuError::DeviceCreation(e.to_string()))?;

        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);

        Ok(())
    }

    /// Check if GPU is initialized
    pub fn is_initialized(&self) -> bool {
        self.device.is_some() && self.queue.is_some()
    }

    /// Get device reference
    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }

    /// Get queue reference
    pub fn queue(&self) -> Option<&Queue> {
        self.queue.as_ref()
    }
}

impl Default for GpuContext {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU render target for offscreen rendering
pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
    pub texture: Option<wgpu::Texture>,
    pub view: Option<TextureView>,
    pub output_buffer: Option<Buffer>,
}

impl RenderTarget {
    /// Create a new render target
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            texture: None,
            view: None,
            output_buffer: None,
        }
    }

    /// Initialize the render target with GPU resources
    pub fn initialize(&mut self, device: &Device, format: TextureFormat) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Target Texture"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create output buffer for reading pixels
        let buffer_size = (self.width * self.height * 4) as u64;
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.texture = Some(texture);
        self.view = Some(view);
        self.output_buffer = Some(output_buffer);
    }

    /// Resize the render target
    pub fn resize(&mut self, device: &Device, format: TextureFormat, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.initialize(device, format);
    }
}

/// Simple 2D renderer using wgpu
pub struct GpuRenderer {
    context: GpuContext,
    render_target: RenderTarget,
    clear_color: wgpu::Color,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            context: GpuContext::new(),
            render_target: RenderTarget::new(width, height),
            clear_color: wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        }
    }
}

/// GPU-related errors
#[derive(Debug)]
pub enum GpuError {
    /// No suitable GPU adapter found
    NoAdapter,
    /// Failed to create device
    DeviceCreation(String),
    /// Shader compilation error
    ShaderCompilation(String),
    /// Buffer creation error
    BufferCreation(String),
    /// Render error
    RenderError(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter => write!(f, "No suitable GPU adapter found"),
            Self::DeviceCreation(e) => write!(f, "Failed to create GPU device: {}", e),
            Self::ShaderCompilation(e) => write!(f, "Shader compilation error: {}", e),
            Self::BufferCreation(e) => write!(f, "Buffer creation error: {}", e),
            Self::RenderError(e) => write!(f, "Render error: {}", e),
        }
    }
}

impl std::error::Error for GpuError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_context_creation() {
        let context = GpuContext::new();
        assert!(!context.is_initialized());
    }

    #[test]
    fn test_render_target_creation() {
        let target = RenderTarget::new(800, 600);
        assert_eq!(target.width, 800);
        assert_eq!(target.height, 600);
        assert!(target.texture.is_none());
    }

    #[test]
    fn test_gpu_renderer_creation() {
        let renderer = GpuRenderer::new(1920, 1080);
        assert_eq!(renderer.render_target.width, 1920);
        assert_eq!(renderer.render_target.height, 1080);
    }
}

