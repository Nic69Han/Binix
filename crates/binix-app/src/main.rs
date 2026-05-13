use eframe::egui;
use binix_core::Result;

fn main() -> Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Binix",
        options,
        Box::new(|_cc| Ok(Box::new(BrowserApp::new()))),
    )
}

struct BrowserApp {
    url: String,
    html: String,
}

impl BrowserApp {
    fn new() -> Self {
        Self {
            url: "https://example.com".into(),
            html: String::new(),
        }
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.url);
            if ui.button("Go").clicked() {
                // Trigger fetch -> parse -> render pipeline
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Content will render here");
        });
    }
}
