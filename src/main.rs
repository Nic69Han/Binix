//! Binix - Ultra-High-Performance Web Browser
//!
//! Entry point for the Binix browser application.

use binix::{NAME, VERSION};
use std::env;

fn main() {
    // Initialize logger
    env_logger::init();

    // Check for CLI mode
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--cli" {
        run_cli_mode();
    } else {
        // Get URL from command line if provided
        let initial_url = if args.len() > 1 && !args[1].starts_with('-') {
            Some(args[1].clone())
        } else {
            None
        };
        run_gui_mode(initial_url);
    }
}

fn run_cli_mode() {
    println!(
        "🚀 {} v{} - Ultra-High-Performance Web Browser",
        NAME, VERSION
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize the browser engine
    let _engine = binix::BrowserEngine::new();

    println!("✅ Browser engine initialized");
    println!("📊 Performance targets:");
    println!(
        "   • Page load: < {}ms",
        binix::performance_targets::PAGE_LOAD_MS
    );
    println!(
        "   • Memory per tab: < {}MB",
        binix::performance_targets::MAX_TAB_MEMORY_MB
    );
    println!(
        "   • Memory reduction: {}% vs Chrome",
        binix::performance_targets::MEMORY_REDUCTION_PERCENT
    );

    println!("\n🔧 CLI mode - use without --cli flag for GUI");
}

fn run_gui_mode(initial_url: Option<String>) {
    println!("🚀 {} v{} - Starting GUI...", NAME, VERSION);

    if let Err(e) = binix::ui::run(initial_url) {
        eprintln!("❌ Failed to start browser: {}", e);
        std::process::exit(1);
    }
}
