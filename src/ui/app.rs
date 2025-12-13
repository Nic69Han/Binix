//! Main browser application using eframe/egui

use super::{TabManager, Theme, UiConfig};
use eframe::egui;

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
            let tabs: Vec<_> = self.tabs.tabs().iter().map(|t| (t.id(), t.title().to_string())).collect();
            let active = self.tabs.active_tab().map(|t| t.id());

            for (id, title) in tabs {
                let is_active = active == Some(id);
                let text = if title.len() > 20 {
                    format!("{}...", &title[..17])
                } else {
                    title
                };

                let button = egui::Button::new(&text)
                    .fill(if is_active {
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
        if let Some(tab) = self.tabs.active_tab() {
            if tab.is_loading() {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            } else if tab.url().is_empty() {
                self.render_new_tab_page(ui);
            } else {
                // TODO: Render actual page content
                ui.centered_and_justified(|ui| {
                    ui.label(format!("Page: {}", tab.url()));
                });
            }
        } else {
            self.render_new_tab_page(ui);
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
                    if ui.selectable_label(self.config.theme == Theme::Light, "Light").clicked() {
                        self.config.theme = Theme::Light;
                    }
                    if ui.selectable_label(self.config.theme == Theme::Dark, "Dark").clicked() {
                        self.config.theme = Theme::Dark;
                    }
                    if ui.selectable_label(self.config.theme == Theme::System, "System").clicked() {
                        self.config.theme = Theme::System;
                    }
                });

                ui.add_space(10.0);
                ui.heading("Zoom");
                ui.add(egui::Slider::new(&mut self.config.default_zoom, 0.5..=2.0).text("Zoom"));

                ui.add_space(10.0);
                ui.heading("Developer");
                if ui.checkbox(&mut self.show_devtools, "Show Developer Tools").changed() {
                    // Toggle devtools
                }
            });
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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


