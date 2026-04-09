//! Example: Simple page scraping with NexScrape
//!
//! ```bash
//! cargo run --example simple_scrape
//! ```

use nexscrape_core::{HttpEngine, EngineConfig, ScrapRequest};

#[tokio::main]
async fn main() -> nexscrape_core::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("nexscrape=info")
        .init();

    println!("🕷️  NexScrape — Simple Scraping Example\n");

    // Create engine with default config
    let engine = HttpEngine::new(EngineConfig {
        concurrency: 4,
        timeout_secs: 15,
        ..Default::default()
    })?;

    // Fetch a page
    let request = ScrapRequest::get("https://example.com");
    let response = engine.execute(request).await?;

    println!("✅ Status: {}", response.status);
    println!("📏 Body size: {} bytes", response.body.len());

    // Parse HTML
    let html = response.html()?;

    // Extract title
    if let Ok(title) = html.select_text("title") {
        println!("📄 Title: {}", title);
    }

    // Extract all links
    let links = html.links()?;
    println!("\n🔗 Links found: {}", links.len());
    for link in &links {
        println!("   → {}", link);
    }

    // Extract meta tags
    let meta = html.meta_tags()?;
    if !meta.is_empty() {
        println!("\n📋 Meta tags:");
        for (key, value) in &meta {
            println!("   {} = {}", key, value);
        }
    }

    println!("\n✨ Done!");
    Ok(())
}
