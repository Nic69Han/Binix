//! Performance benchmarks for Binix browser
//!
//! Target metrics from docs/browser_technical_specification.md:
//! - Page Load Time: <1.5s for typical websites
//! - Memory Usage: 30% less than Chrome baseline
//! - CPU Efficiency: 25% reduction in CPU cycles
//!
//! NOTE: These benchmarks are configured to be lightweight to avoid freezing the system.

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::time::Duration;

/// Configure criterion for lightweight benchmarks
fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10) // Reduced from default 100
        .measurement_time(Duration::from_secs(1)) // 1 second per benchmark
        .warm_up_time(Duration::from_millis(500)) // 500ms warmup
        .without_plots() // Disable plot generation
}

// ============================================================================
// HTML Parsing Benchmarks
// ============================================================================

fn benchmark_html_parsing(c: &mut Criterion) {
    use binix::renderer::HtmlParser;

    let parser = HtmlParser::new();

    let mut group = c.benchmark_group("html_parsing");

    // Simple HTML
    let simple_html = "<html><body><h1>Hello World</h1></body></html>";
    group.throughput(Throughput::Bytes(simple_html.len() as u64));
    group.bench_with_input(BenchmarkId::new("simple", "44B"), simple_html, |b, html| {
        b.iter(|| parser.parse(black_box(html)))
    });

    // Medium HTML (typical page structure)
    let medium_html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Test Page</title>
        </head>
        <body>
            <header><nav><a href="/">Home</a><a href="/about">About</a></nav></header>
            <main>
                <article>
                    <h1>Main Article</h1>
                    <p>First paragraph with some text content.</p>
                    <p>Second paragraph with more content.</p>
                    <ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>
                </article>
                <aside><h2>Sidebar</h2><p>Related content</p></aside>
            </main>
            <footer><p>Copyright 2024</p></footer>
        </body>
        </html>
    "#;
    group.throughput(Throughput::Bytes(medium_html.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("medium", "800B"),
        medium_html,
        |b, html| b.iter(|| parser.parse(black_box(html))),
    );

    // Large HTML (complex page) - reduced from 100 to 20 elements for performance
    let large_html = generate_large_html(20);
    group.throughput(Throughput::Bytes(large_html.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("large", &format!("{}KB", large_html.len() / 1024)),
        &large_html,
        |b, html| b.iter(|| parser.parse(black_box(html))),
    );

    group.finish();
}

fn generate_large_html(num_elements: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><head><title>Large Page</title></head><body>");
    for i in 0..num_elements {
        html.push_str(&format!(
            r#"<div class="item-{0}"><h2>Section {0}</h2><p>Content for section {0} with some text.</p><ul>"#,
            i
        ));
        for j in 0..5 {
            html.push_str(&format!("<li>List item {}-{}</li>", i, j));
        }
        html.push_str("</ul></div>");
    }
    html.push_str("</body></html>");
    html
}

// ============================================================================
// CSS Parsing Benchmarks
// ============================================================================

fn benchmark_css_parsing(c: &mut Criterion) {
    use binix::renderer::CssParser;

    let parser = CssParser::new();

    let mut group = c.benchmark_group("css_parsing");

    // Simple CSS
    let simple_css = "body { margin: 0; padding: 0; }";
    group.throughput(Throughput::Bytes(simple_css.len() as u64));
    group.bench_with_input(BenchmarkId::new("simple", "30B"), simple_css, |b, css| {
        b.iter(|| parser.parse(black_box(css)))
    });

    // Medium CSS (typical stylesheet)
    let medium_css = r#"
        * { box-sizing: border-box; }
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { background: #333; color: white; padding: 15px; }
        .nav a { color: white; text-decoration: none; margin-right: 15px; }
        .main { display: flex; gap: 20px; }
        .sidebar { width: 300px; background: white; padding: 15px; }
        .content { flex: 1; background: white; padding: 15px; }
        .footer { text-align: center; padding: 20px; color: #666; }
        @media (max-width: 768px) { .main { flex-direction: column; } }
    "#;
    group.throughput(Throughput::Bytes(medium_css.len() as u64));
    group.bench_with_input(BenchmarkId::new("medium", "700B"), medium_css, |b, css| {
        b.iter(|| parser.parse(black_box(css)))
    });

    // Large CSS - reduced from 200 to 50 rules for performance
    let large_css = generate_large_css(50);
    group.throughput(Throughput::Bytes(large_css.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("large", &format!("{}KB", large_css.len() / 1024)),
        &large_css,
        |b, css| b.iter(|| parser.parse(black_box(css))),
    );

    group.finish();
}

fn generate_large_css(num_rules: usize) -> String {
    let mut css = String::new();
    for i in 0..num_rules {
        css.push_str(&format!(
            ".class-{0} {{ color: #{0:06x}; padding: {1}px; margin: {2}px; font-size: {3}px; }}\n",
            i,
            i % 50,
            i % 30,
            12 + (i % 10)
        ));
    }
    css
}

// ============================================================================
// Layout Engine Benchmarks
// ============================================================================

fn benchmark_layout(c: &mut Criterion) {
    use binix::renderer::{HtmlParser, LayoutEngine};

    let html_parser = HtmlParser::new();
    let layout_engine = LayoutEngine::new();

    let mut group = c.benchmark_group("layout");

    // Simple layout
    let simple_doc = html_parser
        .parse("<html><body><div><p>Hello</p></div></body></html>")
        .unwrap();
    group.bench_function("simple_layout", |b| {
        b.iter(|| layout_engine.compute(black_box(&simple_doc)))
    });

    // Complex nested layout
    let complex_html = r#"
        <html><body>
            <div><div><div><p>Deeply nested</p></div></div></div>
            <div><span>Inline 1</span><span>Inline 2</span><span>Inline 3</span></div>
            <ul><li>Item 1</li><li>Item 2</li><li>Item 3</li><li>Item 4</li><li>Item 5</li></ul>
        </body></html>
    "#;
    let complex_doc = html_parser.parse(complex_html).unwrap();
    group.bench_function("complex_layout", |b| {
        b.iter(|| layout_engine.compute(black_box(&complex_doc)))
    });

    // Large document layout
    let large_html = generate_large_html(50);
    let large_doc = html_parser.parse(&large_html).unwrap();
    group.bench_function("large_layout", |b| {
        b.iter(|| layout_engine.compute(black_box(&large_doc)))
    });

    group.finish();
}

// ============================================================================
// Memory Pool Benchmarks
// ============================================================================

fn benchmark_object_pool(c: &mut Criterion) {
    use binix::memory::ObjectPool;

    let mut group = c.benchmark_group("object_pool");

    // Pool allocation vs direct allocation
    group.bench_function("pool_get_return", |b| {
        let pool: ObjectPool<Vec<u8>> = ObjectPool::new(100);
        pool.warm(50);

        b.iter(|| {
            let obj = pool.get();
            black_box(obj);
        })
    });

    group.bench_function("direct_allocation", |b| {
        b.iter(|| {
            let obj: Vec<u8> = Vec::with_capacity(1024);
            black_box(obj);
        })
    });

    // Pool with warm-up
    group.bench_function("pool_warmed_100", |b| {
        let pool: ObjectPool<String> = ObjectPool::new(100);
        pool.warm(100);

        b.iter(|| {
            let obj = pool.get();
            black_box(obj);
        })
    });

    group.finish();
}

// ============================================================================
// Compressed Pointer Benchmarks
// ============================================================================

fn benchmark_compressed_pointers(c: &mut Criterion) {
    use binix::memory::CompressedHeap;

    let mut group = c.benchmark_group("compressed_pointers");

    // Compressed heap allocation
    group.bench_function("compressed_alloc", |b| {
        let mut heap: CompressedHeap<u64> = CompressedHeap::new(10000);

        b.iter(|| {
            let ptr = heap.alloc(black_box(42u64));
            black_box(ptr);
        })
    });

    // Compressed pointer dereference
    group.bench_function("compressed_deref", |b| {
        let mut heap: CompressedHeap<u64> = CompressedHeap::new(1000);
        let ptrs: Vec<_> = (0..100).filter_map(|i| heap.alloc(i)).collect();

        b.iter(|| {
            for ptr in &ptrs {
                let val = heap.get(*ptr);
                black_box(val);
            }
        })
    });

    group.finish();
}

// ============================================================================
// Streaming Parser Benchmarks
// ============================================================================

fn benchmark_streaming_parser(c: &mut Criterion) {
    use binix::renderer::StreamingParser;

    let mut group = c.benchmark_group("streaming_parser");

    // Incremental parsing
    group.bench_function("incremental_parse", |b| {
        let chunks = vec![
            "<!DOCTYPE html><html>",
            "<head><title>Test</title></head>",
            "<body><h1>Hello</h1>",
            "<p>Content here</p>",
            "</body></html>",
        ];

        b.iter(|| {
            let mut parser = StreamingParser::new();
            for chunk in &chunks {
                parser.feed_chunk(black_box(chunk));
            }
            parser.finish();
            black_box(parser.get_chunks().len());
        })
    });

    // Large streaming parse
    group.bench_function("streaming_large", |b| {
        let large_chunks: Vec<String> = (0..20)
            .map(|i| format!("<div class='section-{}'><p>Content {}</p></div>", i, i))
            .collect();

        b.iter(|| {
            let mut parser = StreamingParser::new();
            for chunk in &large_chunks {
                parser.feed_chunk(black_box(chunk));
            }
            parser.finish();
            black_box(parser.get_chunks().len());
        })
    });

    group.finish();
}

// ============================================================================
// Dirty Tracking Benchmarks
// ============================================================================

fn benchmark_dirty_tracking(c: &mut Criterion) {
    use binix::renderer::{DirtyTracker, LayoutChange, Rect};

    let mut group = c.benchmark_group("dirty_tracking");

    // Mark dirty regions
    group.bench_function("mark_dirty", |b| {
        let mut tracker = DirtyTracker::new();

        b.iter(|| {
            tracker.mark_dirty(
                black_box(1),
                LayoutChange::ContentChanged,
                Rect::new(0.0, 0.0, 100.0, 100.0),
            );
        })
    });

    // Query dirty state
    group.bench_function("query_dirty", |b| {
        let mut tracker = DirtyTracker::new();
        for i in 0..100 {
            tracker.mark_dirty(i, LayoutChange::ContentChanged, Rect::new(0.0, 0.0, 100.0, 100.0));
        }

        b.iter(|| {
            for i in 0..100 {
                black_box(tracker.is_dirty(i));
            }
        })
    });

    group.finish();
}

// ============================================================================
// Layout Batching Benchmarks
// ============================================================================

fn benchmark_layout_batching(c: &mut Criterion) {
    use binix::renderer::{BatchConfig, LayoutBatcher, LayoutChange, Rect};

    let mut group = c.benchmark_group("layout_batching");

    // Batch operations
    group.bench_function("batch_operations", |b| {
        let config = BatchConfig::default();
        let mut batcher = LayoutBatcher::new(config);

        b.iter(|| {
            for i in 0..50 {
                batcher.queue_change(i, LayoutChange::ContentChanged, Rect::new(0.0, 0.0, 100.0, 100.0));
            }
            let result = batcher.flush();
            black_box(result);
        })
    });

    group.finish();
}

// ============================================================================
// Full Rendering Pipeline Benchmarks
// ============================================================================

fn benchmark_full_pipeline(c: &mut Criterion) {
    use binix::renderer::{CssParser, HtmlParser, LayoutEngine};

    let html_parser = HtmlParser::new();
    let css_parser = CssParser::new();
    let layout_engine = LayoutEngine::new();

    let mut group = c.benchmark_group("full_pipeline");

    // Complete render: HTML -> DOM -> Layout
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Benchmark Page</title></head>
        <body>
            <header><h1>Welcome</h1></header>
            <main>
                <article><h2>Article</h2><p>Content goes here.</p></article>
            </main>
            <footer><p>Footer</p></footer>
        </body>
        </html>
    "#;

    let css = r#"
        body { margin: 0; font-family: sans-serif; }
        header { background: #333; color: white; padding: 20px; }
        main { padding: 20px; }
        footer { text-align: center; }
    "#;

    group.bench_function("html_to_layout", |b| {
        b.iter(|| {
            let doc = html_parser.parse(black_box(html)).unwrap();
            let layout = layout_engine.compute(&doc).unwrap();
            black_box(layout);
        })
    });

    group.bench_function("css_parse", |b| {
        b.iter(|| {
            let stylesheet = css_parser.parse(black_box(css)).unwrap();
            black_box(stylesheet);
        })
    });

    group.bench_function("complete_render", |b| {
        b.iter(|| {
            let doc = html_parser.parse(black_box(html)).unwrap();
            let _stylesheet = css_parser.parse(black_box(css)).unwrap();
            let layout = layout_engine.compute(&doc).unwrap();
            black_box(layout);
        })
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets =
        benchmark_html_parsing,
        benchmark_css_parsing,
        benchmark_layout,
        benchmark_object_pool,
        benchmark_compressed_pointers,
        benchmark_streaming_parser,
        benchmark_dirty_tracking,
        benchmark_layout_batching,
        benchmark_full_pipeline,
}

criterion_main!(benches);
