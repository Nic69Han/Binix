//! Main browser application using eframe/egui

use super::{ElementKind, SharedImageCache, TabManager, Theme, UiConfig, create_shared_cache};
use eframe::egui;
use std::collections::HashMap;

/// Main browser application
pub struct BrowserApp {
    /// UI configuration
    config: UiConfig,
    /// Tab manager
    tabs: TabManager,
    /// URL bar content
    url_input: String,
    /// Show settings panel
    show_settings: bool,
    /// Show developer tools
    show_devtools: bool,
    /// Image cache
    image_cache: SharedImageCache,
    /// Loaded textures for egui
    textures: HashMap<String, egui::TextureHandle>,
}

impl BrowserApp {
    /// Create a new browser application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut tabs = TabManager::new();
        tabs.create_tab();

        Self {
            config: UiConfig::default(),
            tabs,
            url_input: String::new(),
            show_settings: false,
            show_devtools: false,
            image_cache: create_shared_cache(),
            textures: HashMap::new(),
        }
    }

    /// Queue an image for loading
    fn queue_image_load(&mut self, url: String) {
        let mut cache = self.image_cache.lock().unwrap();
        cache.request(&url);
    }

    /// Process pending image loads (called each frame)
    fn process_image_loads(&mut self, ctx: &egui::Context) {
        let pending = {
            let mut cache = self.image_cache.lock().unwrap();
            cache.take_pending()
        };

        for url in pending {
            let cache = self.image_cache.clone();
            let ctx = ctx.clone();
            let url_clone = url.clone();

            // Spawn async task to load image
            std::thread::spawn(move || {
                match load_image_blocking(&url_clone) {
                    Ok(image_data) => {
                        let loaded = super::LoadedImage {
                            data: std::sync::Arc::new(image_data),
                            url: url_clone.clone(),
                            width: 0,
                            height: 0,
                        };
                        let mut cache = cache.lock().unwrap();
                        cache.set_loaded(&url_clone, loaded);
                        ctx.request_repaint();
                    }
                    Err(e) => {
                        let mut cache = cache.lock().unwrap();
                        cache.set_failed(&url_clone, e);
                        ctx.request_repaint();
                    }
                }
            });
        }

        // Convert loaded images to textures
        self.update_textures(ctx);
    }

    /// Update egui textures from loaded images
    fn update_textures(&mut self, ctx: &egui::Context) {
        // Collect images to convert to textures
        let to_load: Vec<(String, egui::ColorImage)> = {
            let cache = self.image_cache.lock().unwrap();
            cache.images.iter()
                .filter(|(url, _)| !self.textures.contains_key(*url))
                .filter_map(|(url, state)| {
                    if let super::ImageState::Loaded(loaded) = state {
                        Some((url.clone(), (*loaded.data).clone()))
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Now create textures without holding the lock
        for (url, image) in to_load {
            let texture = ctx.load_texture(
                &url,
                image,
                egui::TextureOptions::default(),
            );
            self.textures.insert(url, texture);
        }
    }

    /// Render the toolbar
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Navigation buttons
            if ui.button("←").clicked() {
                // TODO: Go back
            }
            if ui.button("→").clicked() {
                // TODO: Go forward
            }
            if ui.button("⟳").clicked() {
                // TODO: Reload
            }
            if ui.button("🏠").clicked() {
                self.url_input = "about:home".to_string();
            }

            // URL bar
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(ui.available_width() - 100.0)
                    .hint_text("Enter URL or search..."),
            );

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.navigate(&self.url_input);
                }
            }

            // Menu button
            if ui.button("☰").clicked() {
                self.show_settings = !self.show_settings;
            }
        });
    }

    /// Render the tab bar
    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let tabs: Vec<_> = self
                .tabs
                .tabs()
                .iter()
                .map(|t| (t.id(), t.title().to_string()))
                .collect();
            let active = self.tabs.active_tab().map(|t| t.id());

            for (id, title) in tabs {
                let is_active = active == Some(id);
                let text = if title.len() > 20 {
                    format!("{}...", &title[..17])
                } else {
                    title
                };

                let button = egui::Button::new(&text).fill(if is_active {
                    ui.style().visuals.selection.bg_fill
                } else {
                    ui.style().visuals.widgets.inactive.bg_fill
                });

                if ui.add(button).clicked() {
                    self.tabs.set_active(id);
                }

                // Close button for each tab
                if ui.small_button("×").clicked() {
                    self.tabs.close_tab(id);
                }

                ui.separator();
            }

            // New tab button
            if ui.button("+").clicked() {
                let id = self.tabs.create_tab();
                self.tabs.set_active(id);
                self.url_input.clear();
            }
        });
    }

    /// Render the content area
    fn render_content(&mut self, ui: &mut egui::Ui) {
        // Poll for content updates
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.poll_content();
        }

        if let Some(tab) = self.tabs.active_tab() {
            if tab.is_loading() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.spinner();
                        ui.label("Loading...");
                    });
                });
            } else if tab.url().is_empty() {
                self.render_new_tab_page(ui);
            } else {
                // Render actual page content
                let content = tab.content();

                if let Some(error) = &content.error {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                } else {
                    self.render_page_content(ui, &content);
                }
            }
        } else {
            self.render_new_tab_page(ui);
        }
    }

    /// Render parsed HTML content with CSS styling
    fn render_page_content(&mut self, ui: &mut egui::Ui, content: &super::PageContent) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                for element in &content.elements {
                    self.render_element(ui, element);
                }
            });
    }

    /// Render a single element with its CSS styles
    fn render_element(&mut self, ui: &mut egui::Ui, element: &super::RenderElement) {
        let style = &element.style;

        // Apply margin (top)
        if style.margin[0] > 0.0 {
            ui.add_space(style.margin[0]);
        }

        // Calculate indent from x position
        let indent = element.bounds.x;

        ui.horizontal(|ui| {
            // Left margin + indent
            ui.add_space(indent + style.margin[3]);

            // Create styled text
            let mut rich_text = egui::RichText::new(&element.text)
                .size(style.font_size);

            // Apply text color
            rich_text = rich_text.color(egui::Color32::from_rgba_unmultiplied(
                style.color[0],
                style.color[1],
                style.color[2],
                style.color[3],
            ));

            // Apply font weight
            if style.font_weight_bold {
                rich_text = rich_text.strong();
            }

            // Apply font style
            if style.font_style_italic {
                rich_text = rich_text.italics();
            }

            // Apply underline
            if style.text_decoration_underline {
                rich_text = rich_text.underline();
            }

            // Render based on element type with background
            match &element.kind {
                ElementKind::Heading1 | ElementKind::Heading2 | ElementKind::Heading3 => {
                    // Render headings with optional background
                    if let Some(bg) = style.background_color {
                        egui::Frame::NONE
                            .fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]))
                            .inner_margin(style.padding[0])
                            .show(ui, |ui| {
                                ui.heading(rich_text);
                            });
                    } else {
                        ui.heading(rich_text);
                    }
                }
                ElementKind::Link => {
                    let link = ui.link(rich_text);
                    if link.clicked() {
                        if let Some(href) = &element.href {
                            self.url_input = href.clone();
                            if let Some(tab) = self.tabs.active_tab_mut() {
                                tab.navigate(href);
                            }
                        }
                    }
                    if link.hovered() {
                        if let Some(href) = &element.href {
                            link.on_hover_text(href);
                        }
                    }
                }
                ElementKind::Code => {
                    // Code with background
                    let bg = style.background_color.unwrap_or([245, 245, 245, 255]);
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]))
                        .inner_margin(style.padding[0])
                        .corner_radius(3)
                        .show(ui, |ui| {
                            ui.label(rich_text.monospace());
                        });
                }
                ElementKind::Paragraph | ElementKind::Text | ElementKind::ListItem => {
                    if let Some(bg) = style.background_color {
                        egui::Frame::NONE
                            .fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]))
                            .inner_margin(style.padding[0])
                            .show(ui, |ui| {
                                ui.label(rich_text);
                            });
                    } else {
                        ui.label(rich_text);
                    }
                }
                ElementKind::Image => {
                    // Try to render actual image if loaded
                    if let Some(src) = &element.src {
                        if let Some(texture) = self.textures.get(src) {
                            // Render loaded image
                            let size = texture.size_vec2();
                            let max_width = ui.available_width().min(600.0);
                            let scale = if size.x > max_width { max_width / size.x } else { 1.0 };
                            ui.image((texture.id(), egui::vec2(size.x * scale, size.y * scale)));
                        } else {
                            // Image not loaded yet, show placeholder and queue loading
                            self.queue_image_load(src.clone());
                            egui::Frame::NONE
                                .fill(egui::Color32::from_rgb(240, 240, 240))
                                .inner_margin(8.0)
                                .corner_radius(4)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label(element.alt.as_deref().unwrap_or("[Loading...]"));
                                    });
                                });
                        }
                    } else {
                        // No src, show placeholder with alt text
                        egui::Frame::NONE
                            .fill(egui::Color32::from_rgb(240, 240, 240))
                            .inner_margin(8.0)
                            .corner_radius(4)
                            .show(ui, |ui| {
                                ui.label(rich_text);
                            });
                    }
                }
                ElementKind::Blockquote => {
                    // Render blockquote with left border
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(250, 250, 250))
                        .inner_margin(egui::Margin {
                            left: 16,
                            right: 8,
                            top: 8,
                            bottom: 8,
                        })
                        .show(ui, |ui| {
                            // Draw left border
                            let rect = ui.available_rect_before_wrap();
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(rect.left() - 12.0, rect.top()),
                                    egui::vec2(4.0, rect.height().max(20.0)),
                                ),
                                0.0,
                                egui::Color32::from_rgb(200, 200, 200),
                            );
                            ui.label(rich_text);
                        });
                }
                ElementKind::Table | ElementKind::TableRow => {
                    // Tables are handled structurally, individual cells are rendered
                    ui.label(rich_text);
                }
                ElementKind::TableCell => {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.label(rich_text);
                        });
                }
                ElementKind::HorizontalRule => {
                    // Render horizontal rule
                    let available_width = ui.available_width();
                    ui.add(egui::Separator::default().horizontal());
                    let _ = available_width; // Suppress warning
                }
                ElementKind::Button => {
                    // Render button
                    let bg = style.background_color.unwrap_or([59, 130, 246, 255]);
                    if ui.add(egui::Button::new(rich_text)
                        .fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]))
                    ).clicked() {
                        // Button click handling would go here
                    }
                }
                ElementKind::Input => {
                    // Render input field placeholder
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(255, 255, 255))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)))
                        .inner_margin(8)
                        .corner_radius(4)
                        .show(ui, |ui| {
                            ui.label(rich_text.weak());
                        });
                }
                ElementKind::Label => {
                    ui.label(rich_text);
                }
            }
        });

        // Apply margin (bottom)
        if style.margin[2] > 0.0 {
            ui.add_space(style.margin[2]);
        }
    }

    /// Render new tab page
    fn render_new_tab_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("🌐 Binix Browser");
            ui.add_space(20.0);
            ui.label("Ultra-high-performance browser written in Rust");
            ui.add_space(40.0);

            // Quick links
            ui.horizontal(|ui| {
                if ui.button("📰 News").clicked() {
                    self.url_input = "https://news.ycombinator.com".to_string();
                }
                if ui.button("🔍 Search").clicked() {
                    self.url_input = "https://duckduckgo.com".to_string();
                }
                if ui.button("📧 Mail").clicked() {
                    self.url_input = "https://mail.google.com".to_string();
                }
            });
        });
    }

    /// Render settings panel
    fn render_settings(&mut self, ctx: &egui::Context) {
        egui::Window::new("Settings")
            .open(&mut self.show_settings)
            .show(ctx, |ui| {
                ui.heading("Appearance");
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    if ui
                        .selectable_label(self.config.theme == Theme::Light, "Light")
                        .clicked()
                    {
                        self.config.theme = Theme::Light;
                    }
                    if ui
                        .selectable_label(self.config.theme == Theme::Dark, "Dark")
                        .clicked()
                    {
                        self.config.theme = Theme::Dark;
                    }
                    if ui
                        .selectable_label(self.config.theme == Theme::System, "System")
                        .clicked()
                    {
                        self.config.theme = Theme::System;
                    }
                });

                ui.add_space(10.0);
                ui.heading("Zoom");
                ui.add(egui::Slider::new(&mut self.config.default_zoom, 0.5..=2.0).text("Zoom"));

                ui.add_space(10.0);
                ui.heading("Developer");
                if ui
                    .checkbox(&mut self.show_devtools, "Show Developer Tools")
                    .changed()
                {
                    // Toggle devtools
                }
            });
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process pending image loads
        self.process_image_loads(ctx);

        // Apply theme
        match self.config.theme {
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
            Theme::System => {} // Use system default
        }

        // Top panel with toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.render_toolbar(ui);
        });

        // Tab bar
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            self.render_tab_bar(ui);
        });

        // Status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(tab) = self.tabs.active_tab() {
                    if tab.is_loading() {
                        ui.spinner();
                        ui.label("Loading...");
                    } else {
                        ui.label(format!("Ready - {}", tab.url()));
                    }
                } else {
                    ui.label("Ready");
                }
            });
        });

        // Developer tools panel
        if self.show_devtools {
            egui::TopBottomPanel::bottom("devtools")
                .resizable(true)
                .min_height(100.0)
                .show(ctx, |ui| {
                    ui.heading("Developer Tools");
                    ui.separator();
                    ui.label("Console, Network, Elements, etc.");
                });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_content(ui);
        });

        // Settings window
        if self.show_settings {
            self.render_settings(ctx);
        }
    }
}

/// Run the browser application
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("Binix Browser"),
        ..Default::default()
    };

    eframe::run_native(
        "Binix Browser",
        options,
        Box::new(|cc| Ok(Box::new(BrowserApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_app_creation() {
        // We can't fully test eframe app without a display,
        // but we can test the underlying structures
        let tabs = TabManager::new();
        assert_eq!(tabs.count(), 0);
    }

    #[test]
    fn test_ui_config_default() {
        let config = UiConfig::default();
        assert_eq!(config.window_width, 1280);
        assert_eq!(config.window_height, 720);
        assert_eq!(config.default_zoom, 1.0);
    }
}

/// Load an image from URL synchronously (blocking)
fn load_image_blocking(url: &str) -> Result<egui::ColorImage, String> {
    // Use reqwest blocking client
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to fetch image: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read image data: {}", e))?;

    super::decode_image(&bytes)
}
