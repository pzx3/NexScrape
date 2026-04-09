//! NexScrape CLI — Command-line web scraping tool.

use clap::{Parser, Subcommand};
use nexscrape_core::{
    HttpEngine, EngineConfig, ScrapRequest, Item,
    storage::export::{Exporter, ExportFormat},
};
use std::collections::HashMap;

#[derive(Parser)]
#[command(
    name = "nex",
    version,
    about = "🕷️ NexScrape — Next-generation web scraping CLI",
    long_about = "NexScrape is a blazing-fast web scraping tool built with Rust.\nUse it to fetch pages, extract data, and export results in seconds."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch a URL and extract data.
    Fetch {
        /// URL to fetch.
        url: String,

        /// CSS selector to extract.
        #[arg(short, long)]
        select: Option<String>,

        /// Output format (json, csv, jsonl).
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path (prints to stdout if not specified).
        #[arg(short, long)]
        output: Option<String>,

        /// Request timeout in seconds.
        #[arg(short, long, default_value = "30")]
        timeout: u64,

        /// Custom User-Agent string.
        #[arg(long)]
        user_agent: Option<String>,
    },

    /// Extract structured data from a URL.
    Extract {
        /// URL to extract from.
        url: String,

        /// Fields to extract in format "name:selector,name:selector".
        #[arg(short, long)]
        fields: String,

        /// Output file path.
        #[arg(short, long)]
        output: Option<String>,

        /// Output format.
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Show information about NexScrape.
    Info,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nexscrape=info".parse()?)
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch {
            url,
            select,
            format,
            output,
            timeout,
            user_agent,
        } => {
            cmd_fetch(url, select, format, output, timeout, user_agent).await?;
        }
        Commands::Extract {
            url,
            fields,
            output,
            format,
        } => {
            cmd_extract(url, fields, output, format).await?;
        }
        Commands::Info => {
            cmd_info();
        }
    }

    Ok(())
}

async fn cmd_fetch(
    url: String,
    select: Option<String>,
    format: String,
    output: Option<String>,
    timeout: u64,
    user_agent: Option<String>,
) -> anyhow::Result<()> {
    let config = EngineConfig {
        timeout_secs: timeout,
        user_agent: user_agent.unwrap_or_else(|| EngineConfig::default().user_agent),
        ..Default::default()
    };

    let engine = HttpEngine::new(config)?;
    let request = ScrapRequest::get(&url);

    eprintln!("🕷️  Fetching: {}", url);

    let response = engine.execute(request).await?;

    eprintln!("✅ Status: {} | Size: {} bytes", response.status, response.body.len());

    if let Some(selector) = select {
        let html = response.html()?;
        let elements = html.select(&selector)?;

        let items: Vec<Item> = elements
            .iter()
            .map(|el| {
                Item::new(&url)
                    .set("text", serde_json::Value::String(el.text()))
                    .set("html", serde_json::Value::String(el.inner_html()))
            })
            .collect();

        let export_format = match format.as_str() {
            "csv" => ExportFormat::Csv,
            "jsonl" => ExportFormat::JsonLines,
            _ => ExportFormat::Json,
        };

        let result = match export_format {
            ExportFormat::Json => Exporter::to_json(&items)?,
            ExportFormat::JsonLines => Exporter::to_jsonl(&items)?,
            ExportFormat::Csv => Exporter::to_csv(&items)?,
        };

        match output {
            Some(path) => {
                std::fs::write(&path, &result)?;
                eprintln!("📁 Saved to: {}", path);
            }
            None => println!("{}", result),
        }
    } else {
        let text = response.text()?;
        match output {
            Some(path) => {
                std::fs::write(&path, &text)?;
                eprintln!("📁 Saved to: {}", path);
            }
            None => println!("{}", text),
        }
    }

    Ok(())
}

async fn cmd_extract(
    url: String,
    fields_str: String,
    output: Option<String>,
    format: String,
) -> anyhow::Result<()> {
    let engine = HttpEngine::new(EngineConfig::default())?;
    let request = ScrapRequest::get(&url);

    eprintln!("🕷️  Extracting from: {}", url);

    let response = engine.execute(request).await?;
    let html = response.html()?;

    // Parse fields: "title:h1,price:.price"
    let mut selectors = HashMap::new();
    for field_def in fields_str.split(',') {
        let parts: Vec<&str> = field_def.splitn(2, ':').collect();
        if parts.len() == 2 {
            selectors.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
        }
    }

    let extracted = html.extract_map(&selectors)?;
    let item = Item {
        fields: extracted.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect(),
        source_url: url.clone(),
    };

    let export_format = match format.as_str() {
        "csv" => ExportFormat::Csv,
        "jsonl" => ExportFormat::JsonLines,
        _ => ExportFormat::Json,
    };

    let result = match export_format {
        ExportFormat::Json => Exporter::to_json(&[item])?,
        ExportFormat::JsonLines => Exporter::to_jsonl(&[item])?,
        ExportFormat::Csv => Exporter::to_csv(&[item])?,
    };

    match output {
        Some(path) => {
            std::fs::write(&path, &result)?;
            eprintln!("📁 Saved to: {}", path);
        }
        None => println!("{}", result),
    }

    Ok(())
}

fn cmd_info() {
    println!(r#"
  🕷️  NexScrape v{}
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Next-generation web scraping engine
  Built with Rust for maximum performance

  Features:
    ⚡ Async HTTP/2 with connection pooling
    🎭 Browser fingerprint rotation
    🔄 Proxy pool with smart rotation
    📊 Export to JSON, CSV, JSONL
    🛡️ Rate limiting & retry policies
    🧹 URL deduplication (Bloom filter)

  Repository: https://github.com/nexscrape/nexscrape
  License: MIT
"#, env!("CARGO_PKG_VERSION"));
}
