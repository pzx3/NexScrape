<p align="center">
  <h1 align="center">🕷️ NexScrape</h1>
  <p align="center">
    <strong>Next-generation high-performance web scraping engine</strong>
  </p>
  <p align="center">
    <a href="#features">Features</a> •
    <a href="#installation">Install</a> •
    <a href="#quick-start">Quick Start</a> •
    <a href="#cli">CLI</a> •
    <a href="#api-reference">API</a> •
    <a href="#architecture">Architecture</a> •
    <a href="#contributing">Contributing</a>
  </p>
  <p align="center">
    <img src="https://img.shields.io/crates/v/nexscrape-core?color=%23e6522c&label=crates.io" alt="crates.io">
    <img src="https://img.shields.io/pypi/v/nexscrape?color=%233776ab&label=PyPI" alt="PyPI">
    <img src="https://img.shields.io/github/license/nexscrape/nexscrape?color=%2332c955" alt="License">
    <img src="https://img.shields.io/github/actions/workflow/status/nexscrape/nexscrape/ci.yml?label=CI" alt="CI">
  </p>
</p>

---

**NexScrape** is a blazing-fast web scraping library with a **Rust core** and **Python bindings**. It combines the raw performance of Rust with the simplicity of Python to deliver a scraping experience that's fast, reliable, and easy to use.

## ⚡ Performance

| Benchmark | Scrapy | Playwright | **NexScrape** |
|-----------|--------|------------|---------------|
| Requests/sec | ~500 | ~50 | **~3,000+** |
| Memory (10 concurrent) | ~200MB | ~800MB | **~30MB** |
| JavaScript support | ❌ | ✅ | ✅ |
| Anti-detection | ❌ | ⚠️ | ✅ |

## ✨ Features

- 🚀 **Blazing fast** — Rust core with async HTTP/2, connection pooling
- 🎭 **Stealth mode** — Browser fingerprint rotation, human behavior simulation
- 🔄 **Smart proxies** — Proxy pool with round-robin, random, and sticky-session rotation
- 🛡️ **Anti-detection** — TLS fingerprinting, CAPTCHA solver integration
- 📊 **Data pipeline** — Schema validation, JSON/CSV/JSONL export
- 🧹 **Deduplication** — Space-efficient Bloom filter
- ⚙️ **Middleware** — Extensible request/response processing pipeline
- 🔁 **Retry & rate limiting** — Exponential backoff, token-bucket rate limiter
- 💾 **Caching** — In-memory LRU cache with TTL
- 🖥️ **CLI tool** — Quick scraping from the terminal

## 📦 Installation

### Rust (CLI & Library)

```bash
# Install CLI
cargo install nexscrape-cli

# Add as dependency
cargo add nexscrape-core
```

### Python

```bash
pip install nexscrape
```

### From Source

```bash
git clone https://github.com/nexscrape/nexscrape.git
cd nexscrape

# Build Rust
cargo build --release

# Build Python bindings
cd nexscrape-python
pip install maturin
maturin develop
```

## 🚀 Quick Start

### Rust

```rust
use nexscrape_core::{fetch, HttpEngine, EngineConfig, ScrapRequest};

#[tokio::main]
async fn main() -> nexscrape_core::Result<()> {
    // Simple one-shot fetch
    let response = fetch("https://example.com").await?;
    let html = response.html()?;

    // Extract title
    let title = html.select_text("title")?;
    println!("Title: {}", title);

    // Extract all links
    let links = html.links()?;
    for link in links {
        println!("Link: {}", link);
    }

    Ok(())
}
```

### Python

```python
import nexscrape as nex

class QuotesSpider(nex.Spider):
    name = "quotes"
    start_urls = ["https://quotes.toscrape.com"]

    config = nex.Config(
        concurrency=16,
        delay=(0.5, 2.0),
    )

    async def parse(self, response):
        for quote in response.css(".quote"):
            yield nex.Item(
                text=quote.css(".text::text").get(),
                author=quote.css(".author::text").get(),
            )

        next_page = response.css("li.next a::attr(href)").get()
        if next_page:
            yield nex.Request(next_page, callback=self.parse)

# Run it
nex.run(QuotesSpider, output="quotes.json")
```

### JavaScript/TypeScript Library (NexScrape Browser)

For interacting with highly dynamic Single Page Applications (SPAs) and executing JavaScript:

```typescript
import { BrowserRuntime } from "@nexscrape/browser";

async function run() {
  const browser = new BrowserRuntime({ headless: true });
  const page = await browser.launch();

  // 1. Visit and Extract Text
  await page.visit("http://localhost:3000");
  const title = await page.runScript(() => document.querySelector('h1')?.textContent);
  console.log("Title:", title);

  // 2. Extract Attribute (Link)
  const link = await page.runScript(() => document.querySelector('a.btn')?.getAttribute('href'));
  console.log("Link:", link);

  // 3. Extract Media (Image/Poster)
  const imageUrl = await page.runScript(() => document.querySelector('img')?.src);
  console.log("Image:", imageUrl);

  await browser.close();
}
run();
```

### Visual Selector Tool (NexPicker)

NexPicker is a powerful visual interface to "pick" elements and generate schemas without writing code.

**Key Features:**
- 🌍 **Arabic Support** — Correctly handles connected Arabic characters in the terminal.
- 🔗 **Structural Paths** — Generates deep, resilient CSS and XPath paths.
- 🛡️ **Single-Tab Mode** — Forces all links to open in the same window (strips `target="_blank"`).
- 🔄 **Persistent Injection** — Stays active across page navigations.

**Launch it:**
```bash
# In nexscrape-picker directory
npx tsx src/cli.ts
```

### 🧪 Test Laboratory

We provide a built-in test site to verify selector accuracy and Arabic support:

```bash
cd test-site
node server.js
# Open http://localhost:3000
```


## 🖥️ CLI

```bash
# Fetch a page and extract data
nex fetch "https://example.com" --select "h1, p" --format json

# Extract structured fields
nex extract "https://news.site.com" \
    --fields "title:h1,date:.date,body:.content" \
    --output article.json

# Show NexScrape info
nex info
```

## 📖 API Reference

### Core Types

#### `HttpEngine`
High-performance async HTTP client with connection pooling.

```rust
let config = EngineConfig {
    concurrency: 50,
    timeout_secs: 30,
    ..Default::default()
};
let engine = HttpEngine::new(config)?;
let response = engine.execute(ScrapRequest::get("https://example.com")).await?;
```

#### `HtmlParser`
HTML parser with CSS selector support.

```rust
let html = response.html()?;
let titles = html.select_all_text("h2.title")?;
let links = html.links()?;
let meta = html.meta_tags()?;
```

#### `Scheduler`
Priority-based request scheduler with deduplication.

```rust
let scheduler = Scheduler::new(SchedulerConfig::default());
scheduler.enqueue(ScrapRequest::get("https://example.com").priority(10)).await;
let next = scheduler.dequeue().await;
```

### Middleware

#### `FingerprintRotator`
Rotates browser fingerprints to avoid detection.

```rust
let rotator = FingerprintRotator::new(10); // rotate every 10 requests
pipeline.add(rotator);
```

#### `ProxyPool`
Manages proxy rotation with health checking.

```rust
let pool = ProxyPool::from_urls(
    vec!["http://proxy1:8080", "socks5://proxy2:1080"],
    RotationStrategy::RoundRobin,
);
pipeline.add(pool);
```

#### `RateLimiter`
Token-bucket rate limiter with per-domain tracking.

```rust
let limiter = RateLimiter::new(2.0, 10, true); // 2 req/s, burst 10, per-domain
pipeline.add(limiter);
```

### Data Export

```rust
use nexscrape_core::storage::export::{Exporter, ExportFormat};

// Export to JSON
Exporter::to_file(&items, "output.json", ExportFormat::Json)?;

// Export to CSV
Exporter::to_file(&items, "output.csv", ExportFormat::Csv)?;

// Export to JSON Lines
Exporter::to_file(&items, "output.jsonl", ExportFormat::JsonLines)?;
```

## 🏗️ Architecture

```
nexscrape/
├── nexscrape-core/          # Rust core engine
│   └── src/
│       ├── engine/           # HTTP client, scheduler
│       ├── middleware/       # Pipeline, fingerprint, proxy, rate limit
│       ├── parser/           # HTML, JSON, schema validation
│       ├── anti_detection/   # Stealth, CAPTCHA, human simulation
│       └── storage/          # Bloom filter, queue, export
├── nexscrape-python/         # Python bindings (PyO3)
│   └── nexscrape/
├── nexscrape-cli/            # CLI tool
├── examples/                 # Usage examples
├── tests/                    # Integration tests
└── docs/                     # Documentation
```

### Data Flow

```
Request → [Middleware Pipeline] → HTTP Engine → Response
                                                  │
                                           Parser Engine
                                                  │
                                           Schema Validation
                                                  │
                                            Data Export
```

## 🧪 Testing

```bash
# Run all Rust tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific module tests
cargo test -p nexscrape-core parser::html

# Run Python tests
cd nexscrape-python
pytest tests/
```

## 🗺️ Roadmap

- [x] **v0.1** — Core HTTP engine, HTML parser, CLI, basic export
- [ ] **v0.5** — Anti-detection, proxy rotation, CAPTCHA integration
- [x] **v0.8** — Browser support (Chromium), JavaScript rendering, Advanced JS DSL
- [x] **v0.9** — NexPicker (Visual Selector Tool) & Schema Generation
- [ ] **v1.0** — Distributed crawling, dashboard, ML schema detection

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Clone & build
git clone https://github.com/nexscrape/nexscrape.git
cd nexscrape
cargo build
cargo test --workspace
```

## ⚖️ License

MIT License — see [LICENSE](LICENSE) for details.

## ⚠️ Disclaimer

NexScrape is a technical tool for web data extraction. Users are responsible for:
- Complying with websites' Terms of Service
- Respecting `robots.txt` directives
- Following data protection regulations (GDPR, CCPA, etc.)
- Not overloading target servers

---

<p align="center">
  Built with 🦀 Rust + 🐍 Python | <a href="https://github.com/nexscrape/nexscrape">GitHub</a>
</p>
