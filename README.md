# 🌐 Binix Browser

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/Nic69Han/Binix/actions/workflows/ci.yml/badge.svg)](https://github.com/Nic69Han/Binix/actions)

**Binix** is an ultra-high-performance web browser written entirely in Rust, designed for speed, security, and efficiency.

## ✨ Features

- 🚀 **High Performance** - Targets 30% less memory usage than Chrome
- 🔒 **Memory Safe** - Built with Rust for guaranteed memory safety
- ⚡ **GPU Accelerated** - Hardware rendering with wgpu (Vulkan/Metal/DX12)
- 🌍 **Modern Standards** - HTML5, CSS3, ES2023+ JavaScript support
- 🔌 **Async I/O** - Non-blocking network operations with Tokio
- 🎨 **Native UI** - Cross-platform interface with egui

## 🏗️ Architecture

```
binix/
├── src/
│   ├── engine/       # Core browser engine
│   ├── renderer/     # HTML/CSS parsing & layout
│   ├── network/      # HTTP client & DNS
│   ├── js_engine/    # JavaScript runtime (Boa)
│   ├── compositor/   # GPU rendering
│   ├── ui/           # User interface
│   └── utils/        # Utilities & error handling
├── tests/            # Integration tests
├── benches/          # Performance benchmarks
└── docs/             # Documentation
```

## 🛠️ Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| HTML Parser | html5ever |
| CSS Parser | cssparser |
| JavaScript | Boa Engine |
| GPU Rendering | wgpu |
| HTTP Client | reqwest |
| Async Runtime | Tokio |
| UI Framework | eframe/egui |

## 📦 Installation

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Linux: `libxcb`, `libxkbcommon` development packages

```bash
# Ubuntu/Debian
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev

# Fedora
sudo dnf install libxcb-devel libxkbcommon-devel
```

### Build from source

```bash
# Clone the repository
git clone https://github.com/Nic69Han/Binix.git
cd Binix

# Build in release mode
cargo build --release

# Run the browser
cargo run --release
```

## 🚀 Usage

```bash
# Start the browser (GUI mode)
cargo run

# Start in CLI mode
cargo run -- --cli
```

## 🧪 Testing

```bash
# Run all tests
cargo nextest run

# Run with coverage
cargo llvm-cov

# Run benchmarks
cargo bench
```

## 📊 Performance Targets

| Metric | Target | vs Chrome |
|--------|--------|-----------|
| Page Load | < 1.5s | 30% faster |
| Memory/Tab | < 150MB | 30% less |
| CPU Usage | - | 25% less |
| Battery Life | - | 20% better |

## 🗺️ Roadmap

### ✅ Phase 1: Foundation (Complete)
- [x] Project structure & CI/CD
- [x] HTML5 parser (html5ever)
- [x] CSS3 parser (cssparser)
- [x] JavaScript engine (Boa)
- [x] GPU compositor (wgpu)
- [x] HTTP client (reqwest)
- [x] Layout engine
- [x] UI framework (egui)
- [x] Event system

### 🔄 Phase 2: Performance (In Progress)
- [ ] Multi-process architecture
- [ ] HTTP/3 with QUIC
- [ ] WebAssembly runtime
- [ ] Memory optimizations

### 📋 Phase 3: Features (Planned)
- [ ] Developer tools
- [ ] Security hardening
- [ ] Extensions API
- [ ] Sync & profiles

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Servo](https://servo.org/) - Inspiration for browser architecture
- [html5ever](https://github.com/servo/html5ever) - HTML parsing
- [Boa](https://github.com/boa-dev/boa) - JavaScript engine
- [wgpu](https://wgpu.rs/) - GPU abstraction
- [egui](https://github.com/emilk/egui) - Immediate mode GUI

---

<p align="center">
  Made with ❤️ in Rust
</p>

