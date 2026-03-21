//! Main browser application using eframe/egui

use super::{ElementKind, SharedImageCache, TabManager, Theme, UiConfig, create_shared_cache};
use crate::devtools::{DevTools, LogLevel, RequestStatus};
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    /// Form field values (keyed by element id or name)
    form_values: HashMap<String, String>,
    /// Checkbox/radio states
    form_checks: HashMap<String, bool>,
    /// Pending form submission URL (to navigate after render)
    pending_form_submit: Option<String>,
    /// Whether to focus the URL bar on next frame
    focus_url_bar: bool,
    /// Developer tools state
    devtools: Arc<Mutex<DevTools>>,
    /// Active devtools tab: 0=Console, 1=Elements, 2=Network, 3=Performance
    devtools_tab: usize,
    /// Console filter text
    console_filter: String,
    /// Console log level filter
    console_level_filter: Option<LogLevel>,
}

impl BrowserApp {
    /// Create a new browser application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut tabs = TabManager::new();
        tabs.create_tab();

        // Check for initial URL from command line
        let initial_url = INITIAL_URL.with(|url| url.borrow_mut().take());

        let url_input = if let Some(ref url) = initial_url {
            // Navigate to the initial URL
            if let Some(tab) = tabs.active_tab_mut() {
                tab.navigate(url);
            }
            url.clone()
        } else {
            String::new()
        };

        Self {
            config: UiConfig::default(),
            tabs,
            url_input,
            show_settings: false,
            show_devtools: false,
            image_cache: create_shared_cache(),
            textures: HashMap::new(),
            form_values: HashMap::new(),
            form_checks: HashMap::new(),
            pending_form_submit: None,
            focus_url_bar: true, // Focus URL bar on startup
            devtools: Arc::new(Mutex::new(DevTools::new())),
            devtools_tab: 0,
            console_filter: String::new(),
            console_level_filter: None,
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
            // Check navigation state
            let can_go_back = self.tabs.active_tab().map(|t| t.can_go_back()).unwrap_or(false);
            let can_go_forward = self.tabs.active_tab().map(|t| t.can_go_forward()).unwrap_or(false);

            // Navigation buttons
            let back_btn = egui::Button::new("←");
            if ui.add_enabled(can_go_back, back_btn).clicked() {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.go_back();
                    self.url_input = tab.url().to_string();
                }
            }

            let forward_btn = egui::Button::new("→");
            if ui.add_enabled(can_go_forward, forward_btn).clicked() {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.go_forward();
                    self.url_input = tab.url().to_string();
                }
            }

            if ui.button("⟳").clicked() {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.reload();
                }
            }

            if ui.button("🏠").clicked() {
                self.url_input = "about:home".to_string();
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.navigate("about:home");
                }
            }

            // URL bar
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.url_input)
                    .desired_width(ui.available_width() - 100.0)
                    .hint_text("Enter URL or search..."),
            );

            // Focus URL bar on startup
            if self.focus_url_bar {
                response.request_focus();
                self.focus_url_bar = false;
            }

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
                let available_width = ui.available_width();
                ui.set_min_width(available_width);

                // Group consecutive inline elements
                let mut i = 0;
                while i < content.elements.len() {
                    let element = &content.elements[i];

                    if element.is_inline {
                        // Collect all consecutive inline elements
                        let mut inline_group: Vec<&super::RenderElement> = vec![element];
                        let mut j = i + 1;
                        while j < content.elements.len() && content.elements[j].is_inline {
                            inline_group.push(&content.elements[j]);
                            j += 1;
                        }

                        // Check alignment of first element in group
                        let align = inline_group.first().map(|e| e.style.text_align).unwrap_or_default();

                        // Apply alignment
                        match align {
                            super::TextAlign::Center => {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        for elem in &inline_group {
                                            self.render_inline_element(ui, elem);
                                        }
                                    });
                                });
                            }
                            super::TextAlign::Right => {
                                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        for elem in &inline_group {
                                            self.render_inline_element(ui, elem);
                                        }
                                    });
                                });
                            }
                            _ => {
                                // Left or Justify - default behavior
                                ui.horizontal_wrapped(|ui| {
                                    for elem in &inline_group {
                                        self.render_inline_element(ui, elem);
                                    }
                                });
                            }
                        }

                        i = j;
                    } else {
                        // Apply block-level alignment
                        let align = element.style.text_align;
                        match align {
                            super::TextAlign::Center => {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    self.render_element(ui, element);
                                });
                            }
                            super::TextAlign::Right => {
                                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                                    self.render_element(ui, element);
                                });
                            }
                            _ => {
                                self.render_element(ui, element);
                            }
                        }
                        i += 1;
                    }
                }
            });
    }

    /// Render an inline element (no line breaks)
    fn render_inline_element(&mut self, ui: &mut egui::Ui, element: &super::RenderElement) {
        use super::DisplayMode;

        let style = &element.style;

        // Skip hidden elements
        if style.display == DisplayMode::None || !style.visible {
            return;
        }

        // Create styled text
        let mut rich_text = egui::RichText::new(&element.text).size(style.font_size);
        rich_text = rich_text.color(egui::Color32::from_rgba_unmultiplied(
            style.color[0], style.color[1], style.color[2], style.color[3],
        ));
        if style.font_weight_bold {
            rich_text = rich_text.strong();
        }

        match &element.kind {
            super::ElementKind::Label => {
                ui.label(rich_text);
            }
            super::ElementKind::Input => {
                let key = element.form_attrs.as_ref()
                    .map(|f| if !f.id.is_empty() { f.id.clone() } else { f.name.clone() })
                    .unwrap_or_else(|| format!("input_{}", element.bounds.y as u32));
                let input_name = element.form_attrs.as_ref()
                    .map(|f| f.name.clone())
                    .unwrap_or_default();
                let placeholder = element.form_attrs.as_ref()
                    .map(|f| f.placeholder.clone())
                    .unwrap_or_default();
                let form_action = element.form_attrs.as_ref()
                    .and_then(|f| f.form_action.clone());
                let value = self.form_values.entry(key).or_insert_with(|| {
                    element.form_attrs.as_ref().map(|f| f.value.clone()).unwrap_or_default()
                });
                let response = ui.add(
                    egui::TextEdit::singleline(value)
                        .hint_text(&placeholder)
                        .desired_width(120.0)
                        .return_key(Some(egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Enter)))
                );

                // Check if the text edit lost focus due to Enter key press
                if response.lost_focus() {
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if enter_pressed {
                        if let Some(action) = form_action {
                            let query_value = value.clone();
                            let encoded_name = url_encode(&input_name);
                            let encoded_value = url_encode(&query_value);
                            let submit_url = if action.contains('?') {
                                format!("{}&{}={}", action, encoded_name, encoded_value)
                            } else {
                                format!("{}?{}={}", action, encoded_name, encoded_value)
                            };
                            log::info!("Form submit via Enter (inline): {}", submit_url);
                            self.pending_form_submit = Some(submit_url);
                        }
                    }
                }
            }
            super::ElementKind::Button => {
                // Get form action if this is a submit button
                let form_action = element.form_attrs.as_ref()
                    .and_then(|f| f.form_action.clone());

                let bg = style.background_color.unwrap_or([59, 130, 246, 255]);
                let btn = egui::Button::new(rich_text)
                    .fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]));
                if ui.add(btn).clicked() {
                    if let Some(action) = form_action {
                        // Collect all form values and build query string
                        let mut query_parts: Vec<String> = Vec::new();
                        for (key, value) in &self.form_values {
                            if !value.is_empty() {
                                query_parts.push(format!("{}={}", url_encode(key), url_encode(value)));
                            }
                        }
                        let submit_url = if query_parts.is_empty() {
                            action
                        } else if action.contains('?') {
                            format!("{}&{}", action, query_parts.join("&"))
                        } else {
                            format!("{}?{}", action, query_parts.join("&"))
                        };
                        self.pending_form_submit = Some(submit_url);
                    }
                }
            }
            super::ElementKind::Checkbox => {
                let key = element.form_attrs.as_ref()
                    .map(|f| if !f.id.is_empty() { f.id.clone() } else { f.name.clone() })
                    .unwrap_or_else(|| format!("cb_{}", element.bounds.y as u32));
                let checked = self.form_checks.entry(key).or_insert_with(|| {
                    element.form_attrs.as_ref().map(|f| f.checked).unwrap_or(false)
                });
                ui.checkbox(checked, "");
            }
            super::ElementKind::Radio => {
                let key = element.form_attrs.as_ref()
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| format!("radio_{}", element.bounds.y as u32));
                let value = element.form_attrs.as_ref()
                    .map(|f| f.value.clone())
                    .unwrap_or_default();
                let current = self.form_values.entry(key).or_insert_with(|| {
                    if element.form_attrs.as_ref().map(|f| f.checked).unwrap_or(false) {
                        value.clone()
                    } else {
                        String::new()
                    }
                });
                ui.radio_value(current, value, "");
            }
            super::ElementKind::Select => {
                let key = element.form_attrs.as_ref()
                    .map(|f| if !f.id.is_empty() { f.id.clone() } else { f.name.clone() })
                    .unwrap_or_else(|| format!("select_{}", element.bounds.y as u32));
                let options: Vec<(String, String)> = element.form_attrs.as_ref()
                    .map(|f| f.options.clone())
                    .unwrap_or_default();
                let current = self.form_values.entry(key.clone()).or_insert_with(|| {
                    options.first().map(|(v, _)| v.clone()).unwrap_or_default()
                });
                let selected = options.iter().find(|(v, _)| v == current).map(|(_, l)| l.as_str()).unwrap_or("Select...");
                egui::ComboBox::from_id_salt(&key).width(120.0).selected_text(selected).show_ui(ui, |ui| {
                    for (v, l) in &options { ui.selectable_value(current, v.clone(), l); }
                });
            }
            super::ElementKind::Link => {
                if ui.link(rich_text).clicked() {
                    if let Some(href) = &element.href {
                        self.url_input = href.clone();
                        if let Some(tab) = self.tabs.active_tab_mut() {
                            tab.navigate(href);
                        }
                    }
                }
            }
            _ => {
                ui.label(rich_text);
            }
        }
    }

    /// Render a flex container with proper flexbox layout
    fn render_flex_container(&mut self, ui: &mut egui::Ui, element: &super::RenderElement) {
        use super::{FlexDirection, JustifyContent, AlignItems};

        let style = &element.style;
        let flex = &style.flex;

        // Create container frame with background and padding
        let mut frame = egui::Frame::NONE
            .inner_margin(egui::Margin {
                left: style.padding[3] as i8,
                right: style.padding[1] as i8,
                top: style.padding[0] as i8,
                bottom: style.padding[2] as i8,
            });

        if let Some(bg) = style.background_color {
            frame = frame.fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]));
        }

        if let Some(bc) = style.border_color {
            frame = frame.stroke(egui::Stroke::new(
                style.border_width[0],
                egui::Color32::from_rgba_unmultiplied(bc[0], bc[1], bc[2], bc[3]),
            ));
        }

        if style.border_radius > 0.0 {
            frame = frame.rounding(style.border_radius);
        }

        frame.show(ui, |ui| {
            // Determine layout direction and spacing
            let is_row = matches!(flex.direction, FlexDirection::Row | FlexDirection::RowReverse);
            let gap = flex.gap;

            if is_row {
                // Horizontal layout
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;

                    // Apply justify-content
                    match flex.justify_content {
                        JustifyContent::Center => {
                            // Center horizontally: add flexible space on both sides
                            ui.add_space(ui.available_width() / 3.0);
                            self.render_flex_children(ui, element, is_row);
                        }
                        JustifyContent::FlexEnd => {
                            // Push to the right
                            ui.add_space(ui.available_width() * 0.8);
                            self.render_flex_children(ui, element, is_row);
                        }
                        JustifyContent::SpaceBetween | JustifyContent::SpaceAround | JustifyContent::SpaceEvenly => {
                            // For space-* we render children with calculated spacing
                            self.render_flex_children_spaced(ui, element, is_row);
                        }
                        _ => {
                            self.render_flex_children(ui, element, is_row);
                        }
                    }
                });
            } else {
                // Vertical layout (column)
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = gap;

                    match flex.justify_content {
                        JustifyContent::Center => {
                            // Center items horizontally within the column
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                self.render_flex_children(ui, element, is_row);
                            });
                        }
                        _ => {
                            self.render_flex_children(ui, element, is_row);
                        }
                    }
                });
            }
        });
    }

    /// Render flex children
    fn render_flex_children(&mut self, ui: &mut egui::Ui, element: &super::RenderElement, _is_row: bool) {
        for child in &element.children {
            self.render_flex_child(ui, child);
        }
    }

    /// Render flex children with space-between/around/evenly
    fn render_flex_children_spaced(&mut self, ui: &mut egui::Ui, element: &super::RenderElement, is_row: bool) {
        use super::JustifyContent;

        let child_count = element.children.len();
        if child_count == 0 {
            return;
        }

        // For space-between, we need to distribute remaining space
        match element.style.flex.justify_content {
            JustifyContent::SpaceBetween if child_count > 1 => {
                // Render first child
                self.render_flex_child(ui, &element.children[0]);

                // Add flexible space between each child
                for child in element.children.iter().skip(1) {
                    if is_row {
                        ui.add_space(ui.available_width() / (child_count - 1) as f32);
                    } else {
                        ui.add_space(8.0);
                    }
                    self.render_flex_child(ui, child);
                }
            }
            _ => {
                // Default: render with gap
                for child in &element.children {
                    self.render_flex_child(ui, child);
                }
            }
        }
    }

    /// Render a single flex child with its background/styling
    fn render_flex_child(&mut self, ui: &mut egui::Ui, child: &super::RenderElement) {
        let style = &child.style;

        // Wrap in a frame if the child has background styling
        if style.background_color.is_some() || style.border_color.is_some() {
            let mut frame = egui::Frame::NONE
                .inner_margin(egui::Margin {
                    left: style.padding[3] as i8,
                    right: style.padding[1] as i8,
                    top: style.padding[0] as i8,
                    bottom: style.padding[2] as i8,
                });

            if let Some(bg) = style.background_color {
                frame = frame.fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]));
            }

            if let Some(bc) = style.border_color {
                frame = frame.stroke(egui::Stroke::new(
                    style.border_width[0],
                    egui::Color32::from_rgba_unmultiplied(bc[0], bc[1], bc[2], bc[3]),
                ));
            }

            if style.border_radius > 0.0 {
                frame = frame.rounding(style.border_radius);
            }

            frame.show(ui, |ui| {
                // Render the child content
                let mut rich_text = egui::RichText::new(&child.text).size(style.font_size);
                rich_text = rich_text.color(egui::Color32::from_rgba_unmultiplied(
                    style.color[0], style.color[1], style.color[2], style.color[3],
                ));
                if style.font_weight_bold {
                    rich_text = rich_text.strong();
                }
                ui.label(rich_text);
            });
        } else {
            self.render_element(ui, child);
        }
    }

    /// Render a single element with its CSS styles
    fn render_element(&mut self, ui: &mut egui::Ui, element: &super::RenderElement) {
        use super::{DisplayMode, FlexDirection, JustifyContent, AlignItems};

        let style = &element.style;

        // Skip hidden elements
        if style.display == DisplayMode::None || !style.visible {
            return;
        }

        // Handle flex containers with children
        if (style.display == DisplayMode::Flex || style.display == DisplayMode::Grid)
            && !element.children.is_empty()
        {
            self.render_flex_container(ui, element);
            return;
        }

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
                    // Get form action if this is a submit button
                    let form_action = element.form_attrs.as_ref()
                        .and_then(|f| f.form_action.clone());

                    // Render button
                    let bg = style.background_color.unwrap_or([59, 130, 246, 255]);
                    let btn = egui::Button::new(rich_text)
                        .fill(egui::Color32::from_rgba_unmultiplied(bg[0], bg[1], bg[2], bg[3]));
                    if ui.add(btn).clicked() {
                        if let Some(action) = form_action {
                            // Collect all form values and build query string
                            let mut query_parts: Vec<String> = Vec::new();
                            for (key, value) in &self.form_values {
                                if !value.is_empty() {
                                    query_parts.push(format!("{}={}", url_encode(key), url_encode(value)));
                                }
                            }
                            let submit_url = if query_parts.is_empty() {
                                action
                            } else if action.contains('?') {
                                format!("{}&{}", action, query_parts.join("&"))
                            } else {
                                format!("{}?{}", action, query_parts.join("&"))
                            };
                            self.pending_form_submit = Some(submit_url);
                        }
                    }
                }
                ElementKind::Input => {
                    // Get form key for this input
                    let input_name = element.form_attrs.as_ref()
                        .map(|f| f.name.clone())
                        .unwrap_or_default();
                    let key = element.form_attrs.as_ref()
                        .map(|f| if !f.id.is_empty() { f.id.clone() } else { f.name.clone() })
                        .unwrap_or_else(|| format!("input_{}", element.bounds.y as u32));

                    let placeholder = element.form_attrs.as_ref()
                        .map(|f| f.placeholder.clone())
                        .unwrap_or_default();

                    // Get form action if available
                    let form_action = element.form_attrs.as_ref()
                        .and_then(|f| f.form_action.clone());

                    // Get or create value
                    let value = self.form_values.entry(key.clone()).or_insert_with(|| {
                        element.form_attrs.as_ref().map(|f| f.value.clone()).unwrap_or_default()
                    });

                    // Calculate input width - use bounds if available, otherwise use available width
                    let input_width = if element.bounds.width > 50.0 {
                        element.bounds.width.min(ui.available_width() - 20.0).max(200.0)
                    } else {
                        // For search inputs (like Google), use most of available width
                        (ui.available_width() * 0.7).max(300.0).min(600.0)
                    };

                    // Render input with styled frame
                    let response = egui::Frame::NONE
                        .fill(egui::Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)))
                        .corner_radius(20.0)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::singleline(value)
                                    .hint_text(&placeholder)
                                    .desired_width(input_width - 24.0)
                                    .frame(false)
                                    .return_key(Some(egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Enter)))
                            )
                        }).inner;

                    // Check if the text edit lost focus due to Enter key press
                    if response.lost_focus() {
                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if enter_pressed {
                            if let Some(action) = form_action.clone() {
                                // Build form submission URL with query parameters
                                let query_value = value.clone();
                                let encoded_name = url_encode(&input_name);
                                let encoded_value = url_encode(&query_value);
                                let submit_url = if action.contains('?') {
                                    format!("{}&{}={}", action, encoded_name, encoded_value)
                                } else {
                                    format!("{}?{}={}", action, encoded_name, encoded_value)
                                };
                                log::info!("Form submit via Enter: {}", submit_url);
                                self.pending_form_submit = Some(submit_url);
                            }
                        }
                    }
                }
                ElementKind::Textarea => {
                    let key = element.form_attrs.as_ref()
                        .map(|f| if !f.id.is_empty() { f.id.clone() } else { f.name.clone() })
                        .unwrap_or_else(|| format!("textarea_{}", element.bounds.y as u32));

                    let placeholder = element.form_attrs.as_ref()
                        .map(|f| f.placeholder.clone())
                        .unwrap_or_default();

                    let value = self.form_values.entry(key).or_insert_with(|| {
                        element.form_attrs.as_ref().map(|f| f.value.clone()).unwrap_or_default()
                    });

                    ui.add(
                        egui::TextEdit::multiline(value)
                            .hint_text(&placeholder)
                            .desired_width(300.0)
                            .desired_rows(3)
                    );
                }
                ElementKind::Select => {
                    let key = element.form_attrs.as_ref()
                        .map(|f| if !f.id.is_empty() { f.id.clone() } else { f.name.clone() })
                        .unwrap_or_else(|| format!("select_{}", element.bounds.y as u32));

                    let options: Vec<(String, String)> = element.form_attrs.as_ref()
                        .map(|f| f.options.clone())
                        .unwrap_or_default();

                    let current = self.form_values.entry(key.clone()).or_insert_with(|| {
                        options.first().map(|(v, _)| v.clone()).unwrap_or_default()
                    });

                    let selected_label = options.iter()
                        .find(|(v, _)| v == current)
                        .map(|(_, l)| l.as_str())
                        .unwrap_or("Select...");

                    egui::ComboBox::from_id_salt(&key)
                        .width(200.0)
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            for (value, label) in &options {
                                ui.selectable_value(current, value.clone(), label);
                            }
                        });
                }
                ElementKind::Checkbox => {
                    let key = element.form_attrs.as_ref()
                        .map(|f| if !f.id.is_empty() { f.id.clone() } else { f.name.clone() })
                        .unwrap_or_else(|| format!("cb_{}", element.bounds.y as u32));

                    let checked = self.form_checks.entry(key).or_insert_with(|| {
                        element.form_attrs.as_ref().map(|f| f.checked).unwrap_or(false)
                    });

                    ui.checkbox(checked, "");
                }
                ElementKind::Radio => {
                    let key = element.form_attrs.as_ref()
                        .map(|f| f.name.clone())
                        .unwrap_or_else(|| format!("radio_{}", element.bounds.y as u32));
                    let value = element.form_attrs.as_ref()
                        .map(|f| f.value.clone())
                        .unwrap_or_default();

                    let current = self.form_values.entry(key).or_insert_with(|| {
                        if element.form_attrs.as_ref().map(|f| f.checked).unwrap_or(false) {
                            value.clone()
                        } else {
                            String::new()
                        }
                    });

                    ui.radio_value(current, value.clone(), "");
                }
                ElementKind::Label => {
                    ui.label(rich_text);
                }
                ElementKind::Form => {
                    // Forms are containers, handled by children
                }
                ElementKind::Container => {
                    // Containers are handled by render_flex_container
                    // This shouldn't be reached, but handle gracefully
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
            ui.add_space(80.0);
            ui.label(egui::RichText::new("🌐").size(48.0));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Binix").size(32.0).strong());
            ui.label(egui::RichText::new("Ultra-high-performance browser · Rust")
                .size(13.0)
                .color(egui::Color32::GRAY));
            ui.add_space(32.0);

            // Quick navigation buttons
            ui.horizontal_wrapped(|ui| {
                let quick_links = [
                    ("📰", "Hacker News",  "https://news.ycombinator.com"),
                    ("🔍", "DuckDuckGo",   "https://duckduckgo.com"),
                    ("🦀", "crates.io",    "https://crates.io"),
                    ("📖", "MDN Docs",     "https://developer.mozilla.org"),
                    ("🐙", "GitHub",       "https://github.com"),
                    ("🌍", "Wikipedia",   "https://wikipedia.org"),
                ];
                for (icon, label, url) in quick_links {
                    let btn = egui::Button::new(
                        egui::RichText::new(format!("{icon} {label}")).size(13.0)
                    ).min_size(egui::vec2(120.0, 36.0));
                    if ui.add(btn).clicked() {
                        self.url_input = url.to_string();
                        if let Some(tab) = self.tabs.active_tab_mut() {
                            tab.navigate(url);
                        }
                    }
                }
            });

            ui.add_space(40.0);
            ui.separator();
            ui.add_space(12.0);

            // Keyboard shortcuts reference
            ui.label(egui::RichText::new("Keyboard Shortcuts").strong().size(12.0).color(egui::Color32::GRAY));
            ui.add_space(6.0);
            egui::Grid::new("shortcuts")
                .num_columns(2)
                .spacing([24.0, 4.0])
                .show(ui, |ui| {
                    let shortcuts = [
                        ("Ctrl+T",  "New tab"),
                        ("Ctrl+W",  "Close tab"),
                        ("Ctrl+R",  "Reload"),
                        ("Ctrl+L",  "Focus address bar"),
                        ("Alt+←",   "Back"),
                        ("Alt+→",   "Forward"),
                        ("F12",     "Developer tools"),
                        ("Ctrl++/-","Zoom in/out"),
                    ];
                    for (key, desc) in shortcuts {
                        ui.label(egui::RichText::new(key).monospace().size(11.0)
                            .color(egui::Color32::from_rgb(180, 180, 220)));
                        ui.label(egui::RichText::new(desc).size(11.0).color(egui::Color32::GRAY));
                        ui.end_row();
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

    /// Feed JS console output from a page load into the DevTools console
    pub fn push_console_messages(&self, logs: &[String], errors: &[String]) {
        if let Ok(dt) = self.devtools.lock() {
            if let Ok(mut console) = dt.console.lock() {
                for msg in logs {
                    console.log(msg);
                }
                for err in errors {
                    console.error(err);
                }
            }
        }
    }

    /// Render the full DevTools panel
    fn render_devtools(&mut self, ui: &mut egui::Ui) {
        // ── Tab bar ──────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            let tabs = ["🖥 Console", "🔎 Elements", "🌐 Network", "⏱ Performance"];
            for (i, label) in tabs.iter().enumerate() {
                let selected = self.devtools_tab == i;
                let btn = egui::Button::new(*label)
                    .fill(if selected {
                        ui.style().visuals.selection.bg_fill
                    } else {
                        ui.style().visuals.widgets.inactive.bg_fill
                    });
                if ui.add(btn).clicked() {
                    self.devtools_tab = i;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✖ Close").clicked() {
                    self.show_devtools = false;
                }
                if ui.small_button("🗑 Clear").clicked() {
                    if let Ok(dt) = self.devtools.lock() {
                        dt.clear_all();
                    }
                }
            });
        });
        ui.separator();

        match self.devtools_tab {
            0 => self.render_devtools_console(ui),
            1 => self.render_devtools_elements(ui),
            2 => self.render_devtools_network(ui),
            3 => self.render_devtools_performance(ui),
            _ => {}
        }
    }

    /// Console tab
    fn render_devtools_console(&mut self, ui: &mut egui::Ui) {
        // Filter bar
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.console_filter);
            ui.separator();
            for (level, label, color) in [
                (None, "All", egui::Color32::GRAY),
                (Some(LogLevel::Log), "Log", egui::Color32::WHITE),
                (Some(LogLevel::Warn), "Warn", egui::Color32::YELLOW),
                (Some(LogLevel::Error), "Error", egui::Color32::RED),
            ] {
                let active = self.console_level_filter == level;
                let btn = egui::Button::new(egui::RichText::new(label).color(color))
                    .fill(if active {
                        ui.style().visuals.selection.bg_fill
                    } else {
                        egui::Color32::TRANSPARENT
                    });
                if ui.add(btn).clicked() {
                    self.console_level_filter = level;
                }
            }
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let messages = {
                    if let Ok(dt) = self.devtools.lock() {
                        if let Ok(console) = dt.console.lock() {
                            console.messages().cloned().collect::<Vec<_>>()
                        } else { vec![] }
                    } else { vec![] }
                };

                // Also show JS output from current tab
                let tab_logs = self.tabs.active_tab()
                    .map(|t| t.content().console_output)
                    .unwrap_or_default();

                let filter = self.console_filter.to_lowercase();

                // Render tab logs first (current page JS output)
                for log in &tab_logs {
                    if !filter.is_empty() && !log.to_lowercase().contains(&filter) { continue; }
                    if matches!(self.console_level_filter, Some(LogLevel::Warn) | Some(LogLevel::Error)) { continue; }
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(180, 180, 180), "►");
                        ui.label(egui::RichText::new(log).monospace().size(12.0));
                    });
                }

                // Render DevTools console messages
                for msg in &messages {
                    if !filter.is_empty() && !msg.message.to_lowercase().contains(&filter) { continue; }
                    if let Some(lvl) = self.console_level_filter {
                        if msg.level != lvl { continue; }
                    }
                    let (icon, color) = match msg.level {
                        LogLevel::Warn  => ("⚠", egui::Color32::YELLOW),
                        LogLevel::Error => ("✖", egui::Color32::RED),
                        LogLevel::Info  => ("ℹ", egui::Color32::LIGHT_BLUE),
                        LogLevel::Debug => ("🐛", egui::Color32::GRAY),
                        _               => ("►", egui::Color32::WHITE),
                    };
                    ui.horizontal(|ui| {
                        ui.colored_label(color, icon);
                        ui.label(egui::RichText::new(&msg.message).monospace().size(12.0).color(color));
                    });
                }

                if tab_logs.is_empty() && messages.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("No console output").color(egui::Color32::GRAY).italics());
                    });
                }
            });
    }

    /// Elements (DOM) tab
    fn render_devtools_elements(&mut self, ui: &mut egui::Ui) {
        let content = self.tabs.active_tab()
            .map(|t| t.content())
            .unwrap_or_default();

        if content.elements.is_empty() {
            ui.label(egui::RichText::new("No page loaded").color(egui::Color32::GRAY).italics());
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.label(egui::RichText::new(format!(
                    "<html> → {} rendered elements", content.elements.len()
                )).monospace().size(12.0));
                ui.separator();

                for (i, elem) in content.elements.iter().enumerate().take(200) {
                    let tag = format!("{:?}", elem.kind)
                        .to_lowercase()
                        .replace("heading1", "h1")
                        .replace("heading2", "h2")
                        .replace("heading3", "h3");

                    let preview = if elem.text.len() > 50 {
                        format!("{}…", &elem.text[..47])
                    } else {
                        elem.text.clone()
                    };

                    let color = match elem.kind {
                        super::ElementKind::Link => egui::Color32::LIGHT_BLUE,
                        super::ElementKind::Heading1 | super::ElementKind::Heading2 | super::ElementKind::Heading3
                            => egui::Color32::from_rgb(220, 120, 60),
                        _ => egui::Color32::from_rgb(100, 180, 100),
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{:>3}", i)).size(11.0).color(egui::Color32::GRAY));
                        ui.label(egui::RichText::new(format!("<{}>", tag)).monospace().size(12.0).color(color));
                        ui.label(egui::RichText::new(&preview).size(12.0));
                    });
                }

                if content.elements.len() > 200 {
                    ui.label(egui::RichText::new(format!(
                        "… {} more elements (showing first 200)", content.elements.len() - 200
                    )).color(egui::Color32::GRAY).italics());
                }
            });
    }

    /// Network tab
    fn render_devtools_network(&mut self, ui: &mut egui::Ui) {
        let requests = {
            if let Ok(dt) = self.devtools.lock() {
                if let Ok(ni) = dt.network_inspector.lock() {
                    ni.requests().cloned().collect::<Vec<_>>()
                } else { vec![] }
            } else { vec![] }
        };

        if requests.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("No network requests recorded").color(egui::Color32::GRAY).italics());
                ui.label(egui::RichText::new("Navigate to a page to see requests").size(11.0).color(egui::Color32::DARK_GRAY));
            });
            return;
        }

        // Header
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Method").strong().size(11.0));
            ui.add_space(40.0);
            ui.label(egui::RichText::new("Status").strong().size(11.0));
            ui.add_space(30.0);
            ui.label(egui::RichText::new("URL").strong().size(11.0));
        });
        ui.separator();

        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            for req in &requests {
                let status_color = match req.status {
                    RequestStatus::Complete => egui::Color32::GREEN,
                    RequestStatus::Failed   => egui::Color32::RED,
                    _                                         => egui::Color32::YELLOW,
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(req.method.as_str()).monospace().size(11.0));
                    ui.colored_label(status_color, format!("{}", req.status_code.unwrap_or(0)));
                    let url_short = if req.url.len() > 80 { &req.url[..77] } else { &req.url };
                    ui.label(egui::RichText::new(url_short).monospace().size(11.0));
                });
            }
        });
    }

    /// Performance tab
    fn render_devtools_performance(&mut self, ui: &mut egui::Ui) {
        // Collect metrics: profiler data + tab load_time_ms
        let mut metrics: Vec<(String, f32)> = {
            if let Ok(dt) = self.devtools.lock() {
                if let Ok(p) = dt.profiler.lock() {
                    p.summary().into_iter().map(|(k, v)| (k.as_str().to_string(), v as f32)).collect()
                } else { vec![] }
            } else { vec![] }
        };

        // Add live data from active tab
        if let Some(tab) = self.tabs.active_tab() {
            let content = tab.content();
            if let Some(ms) = content.load_time_ms {
                metrics.push(("Page Load".to_string(), ms as f32));
                metrics.push(("Elements Rendered".to_string(), content.elements.len() as f32));
                metrics.push(("JS Errors".to_string(), content.js_errors.len() as f32));
            }
        }

        ui.add_space(8.0);

        if metrics.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("No performance data yet").color(egui::Color32::GRAY).italics());
                ui.label(egui::RichText::new("Navigate to a page to see metrics").size(11.0).color(egui::Color32::DARK_GRAY));
            });
        } else {
            egui::Grid::new("perf_grid")
                .num_columns(3)
                .striped(true)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Metric").strong());
                    ui.label(egui::RichText::new("Value").strong());
                    ui.label(egui::RichText::new("Target").strong());
                    ui.end_row();
                    for (name, value) in &metrics {
                        let (target, unit) = match name.as_str() {
                            "Page Load" | "Load" | "LoadComplete" => (1500.0_f32, "ms"),
                            "TTFB" => (200.0, "ms"),
                            "FCP" | "FirstContentfulPaint" => (1800.0, "ms"),
                            "LCP" | "LargestContentfulPaint" => (2500.0, "ms"),
                            "TTI" | "TimeToInteractive" => (3800.0, "ms"),
                            "Elements Rendered" | "JS Errors" => (f32::MAX, ""),
                            _ => (f32::MAX, "ms"),
                        };
                        let color = if target < f32::MAX {
                            if *value <= target { egui::Color32::GREEN } else { egui::Color32::RED }
                        } else {
                            egui::Color32::WHITE
                        };
                        ui.label(egui::RichText::new(name));
                        ui.colored_label(color, if unit.is_empty() {
                            format!("{:.0}", value)
                        } else {
                            format!("{:.0}{}", value, unit)
                        });
                        if target < f32::MAX {
                            ui.label(egui::RichText::new(format!("< {:.0}ms", target)).color(egui::Color32::GRAY));
                        } else {
                            ui.label("-");
                        }
                        ui.end_row();
                    }
                });
        }
    }


impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process pending image loads
        self.process_image_loads(ctx);

        // ── Keyboard shortcuts ───────────────────────────────────────────────
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl;
            let alt  = i.modifiers.alt;
            // Ctrl+T — new tab
            if ctrl && i.key_pressed(egui::Key::T) {
                let id = self.tabs.create_tab();
                self.tabs.set_active(id);
                self.url_input.clear();
                self.focus_url_bar = true;
            }
            // Ctrl+W — close active tab
            if ctrl && i.key_pressed(egui::Key::W) {
                if let Some(tab) = self.tabs.active_tab() {
                    let id = tab.id();
                    self.tabs.close_tab(id);
                }
            }
            // Ctrl+R or F5 — reload
            if (ctrl && i.key_pressed(egui::Key::R)) || i.key_pressed(egui::Key::F4) {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.reload();
                }
            }
            // Ctrl+L — focus address bar
            if ctrl && i.key_pressed(egui::Key::L) {
                self.focus_url_bar = true;
            }
            // Alt+Left — back
            if alt && i.key_pressed(egui::Key::ArrowLeft) {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    if tab.can_go_back() {
                        tab.go_back();
                        self.url_input = tab.url().to_string();
                    }
                }
            }
            // Alt+Right — forward
            if alt && i.key_pressed(egui::Key::ArrowRight) {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    if tab.can_go_forward() {
                        tab.go_forward();
                        self.url_input = tab.url().to_string();
                    }
                }
            }
            // F12 — toggle DevTools
            if i.key_pressed(egui::Key::F4) {
                self.show_devtools = !self.show_devtools;
            }
            // Escape — close DevTools if open
            if i.key_pressed(egui::Key::Escape) && self.show_devtools {
                self.show_devtools = false;
            }
        });

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
                        let content = tab.content();
                        let load_info = content.load_time_ms
                            .map(|ms| format!(" ({ms}ms)"))
                            .unwrap_or_default();
                        ui.label(format!("✓ Ready{load_info} — {}", tab.url()));
                        // Show element count
                        if !content.elements.is_empty() {
                            ui.separator();
                            ui.label(egui::RichText::new(
                                format!("{} elements", content.elements.len())
                            ).color(egui::Color32::GRAY).size(11.0));
                        }
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
                .min_height(200.0)
                .default_height(280.0)
                .show(ctx, |ui| {
                    self.render_devtools(ui);
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

        // Process pending form submissions
        if let Some(submit_url) = self.pending_form_submit.take() {
            if let Some(tab) = self.tabs.active_tab_mut() {
                self.url_input = submit_url.clone();
                tab.navigate(&submit_url);
            }
        }
    }
}

/// Run the browser application with an optional initial URL
pub fn run(initial_url: Option<String>) -> eframe::Result<()> {
    // Store initial URL in a thread-local or use lazy_static
    INITIAL_URL.with(|url| {
        *url.borrow_mut() = initial_url;
    });

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

thread_local! {
    static INITIAL_URL: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
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

/// URL-encode a string for use in query parameters
fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            ' ' => {
                result.push('+');
            }
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}
